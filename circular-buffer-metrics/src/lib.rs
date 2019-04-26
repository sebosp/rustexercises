//! Exports the TimeSeries class
//! The TimeSeries is a circular buffer that contains an entry per epoch
//! at different granularities. It is maintained as a Vec<(u64, T)> where
//! T is a metric. Since metrics will overwrite the contents of the array
//! partially, the start of the metrics and the end of the metrics are
//! maintained as two separate indexes. This allows the vector to shrink
//! and rotate without relocation of memory or shifting of the vector.

// TODO:
// - Move to the config.yaml
// -- The yaml should drive an array of activity dashboards
// -- The dashboards should be toggable, some key combination
// -- When activated on toggle it could blur a portion of the screen
// -- derive builder
// -- Use prometheus queries instead of our own aggregation/etc.

extern crate futures;
extern crate hyper;
extern crate num_traits;
extern crate serde_json;
extern crate tokio_core;
// use crate::term::color::Rgb;
// use crate::term::SizeInfo;
use num_traits::*;
use std::time::UNIX_EPOCH;

use futures::{Future, Stream};
use hyper::Client;
use serde_json::Value;
use std::io;
use tokio_core::reactor::Core;

/// `MissingValuesPolicy` provides several ways to deal with missing values
/// when drawing the Metric
#[derive(Debug, Clone)]
pub enum MissingValuesPolicy<T>
where
    T: Num + Clone + Copy,
{
    Zero,
    One,
    First,
    Last,
    Fixed(T),
    Avg,
    Max,
    Min,
}

impl<T> Default for MissingValuesPolicy<T>
where
    T: Num + Clone + Copy,
{
    fn default() -> MissingValuesPolicy<T> {
        MissingValuesPolicy::Zero
    }
}

/// `ValueCollisionPolicy` handles collisions when several values are collected
/// for the same time unit, allowing for overwriting, incrementing, etc.
#[derive(Debug, Clone)]
pub enum ValueCollisionPolicy {
    Overwrite,
    Increment,
    Decrement,
    Ignore,
}

impl Default for ValueCollisionPolicy {
    fn default() -> ValueCollisionPolicy {
        ValueCollisionPolicy::Increment
    }
}

/// `TimeSeriesStats` contains statistics about the current TimeSeries
#[derive(Debug, Clone)]
pub struct TimeSeriesStats<T>
where
    T: Num + Clone + Copy,
{
    max: T,
    min: T,
    avg: T, // Calculation may lead to overflow
    first: T,
    last: T,
    count: usize,
    sum: T, // May overflow
    is_dirty: bool,
}

impl<T> Default for TimeSeriesStats<T>
where
    T: Num + Clone + Copy,
{
    fn default() -> TimeSeriesStats<T> {
        TimeSeriesStats {
            max: T::zero(),
            min: T::zero(),
            avg: T::zero(),
            first: T::zero(),
            last: T::zero(),
            count: 0usize,
            sum: T::zero(),
            is_dirty: false,
        }
    }
}
/// `TimeSeries` contains a vector of tuple (epoch, Option<value>)
/// The vector behaves as a circular buffer to avoid shifting values.
/// The circular buffer may be invalidated partially, for example when too much
/// time has passed without metrics, the vecotr is allowed to shrink without
/// memory rellocation, this is achieved by using two indexes for the first
/// and last item.
#[derive(Clone)]
pub struct TimeSeries<T>
where
    T: Num + Clone + Copy,
{
    /// Capture events through time
    /// Contains one entry per time unit
    pub metrics: Vec<(u64, Option<T>)>,

    /// Number of items to store in our metrics vec
    pub metrics_capacity: usize,

    /// Stats for the TimeSeries
    pub metric_stats: TimeSeriesStats<T>,

    /// Useful for records that do not increment but rather are a fixed
    /// or absolute value recorded at a given time
    pub collision_policy: ValueCollisionPolicy,

    /// Missing values can be set to zero
    /// to show where the 1 task per core is
    pub missing_values_policy: MissingValuesPolicy<T>,

    /// The first item in the circular buffer
    pub first_idx: usize,

    /// The last item in the circular buffer
    pub last_idx: usize,

    /// The circular buffer has two indexes, if the start and end
    /// indexes are the same, then the buffer is full or has one item
    /// By knowing the active_items in advance we know which situation is true
    pub active_items: usize,
}
pub struct IterTimeSeries<'a, T: 'a>
where
    T: Num + Clone + Copy,
{
    /// The reference to the TimeSeries struct to iterate over.
    inner: &'a TimeSeries<T>,
    /// The current position state
    pos: usize,
    /// The current item number, to be compared with the active_items
    current_item: usize,
}

pub struct TimeSeriesChart<T>
where
    T: Num + Clone + Copy,
{
    /// The metrics shown at a given time
    pub metrics: TimeSeries<T>,

    /// A marker line to indicate a reference point, for example for load
    /// to show where the 1 loadavg is, or to show disk capacity
    pub metric_reference: Option<T>,

    /// The offset in which the activity line should be drawn
    pub x_offset: f32,

    /// The width of the activity chart/histogram
    pub width: f32,

    /// The height of the activity line region
    pub metrics_height: f32,

    /// The spacing between the activity line segments, could be renamed to line length
    pub tick_spacing: f32,

    /// The color of the activity_line
    pub color: (f32, f32, f32),

    /// The transparency of the activity line
    pub alpha: f32,

    /// The opengl representation of the activity levels
    /// Contains twice as many items because it's x,y
    pub metrics_opengl_vecs: Vec<f32>,

    /// The opengl representation of the activity levels
    /// Contains twice as many items because it's x,y
    pub marker_opengl_vecs: Vec<f32>,
}

impl<T> Default for TimeSeries<T>
where
    T: Num + Clone + Copy,
{
    /// `new` returns the default
    fn default() -> TimeSeries<T> {
        // This leads to 5 mins of metrics to show by default.
        let default_capacity = 300usize;
        TimeSeries {
            metrics_capacity: default_capacity,
            metrics: Vec::with_capacity(default_capacity),
            metric_stats: TimeSeriesStats::default(),
            collision_policy: ValueCollisionPolicy::default(),
            missing_values_policy: MissingValuesPolicy::default(),
            first_idx: 0,
            last_idx: 0,
            active_items: 0,
        }
    }
}
impl<T> TimeSeries<T>
where
    T: Num + Clone + Copy,
{
    /// `with_capacity` builder changes the amount of metrics in the vec
    pub fn with_capacity(self, n: usize) -> TimeSeries<T> {
        let mut new_self = self;
        new_self.metrics = Vec::with_capacity(n);
        new_self.metrics_capacity = n;
        new_self
    }

    /// `with_missing_values_policy` receives a String and returns
    /// a MissingValuesPolicy, TODO: the "Fixed" value is not implemented.
    pub fn with_missing_values_policy(mut self, policy_type: String) -> TimeSeries<T> {
        self.missing_values_policy = match policy_type.as_ref() {
            "zero" => MissingValuesPolicy::Zero,
            "one" => MissingValuesPolicy::One,
            "min" => MissingValuesPolicy::Min,
            "max" => MissingValuesPolicy::Max,
            "last" => MissingValuesPolicy::Last,
            "avg" => MissingValuesPolicy::Avg,
            "first" => MissingValuesPolicy::First,
            _ => {
                // TODO: Implement FromStr somehow
                MissingValuesPolicy::Zero
            }
        };
        self
    }

    /// `calculate_stats` Checks if stats need to be updated for the current
    /// metrics
    pub fn calculate_stats(&mut self)
    where
        T: Num + Clone + Copy + PartialOrd + Bounded + FromPrimitive,
    {
        // Recalculating seems to be necessary because we are constantly
        // moving items out of the Vec<> so our cache can easily get out of
        // sync
        let mut max_activity_value = T::zero();
        let mut min_activity_value = T::max_value();
        let mut sum_activity_values = T::zero();
        let mut filled_metrics = 0usize;
        for entry in self.iter() {
            if let Some(metric) = entry.1 {
                if metric > max_activity_value {
                    max_activity_value = metric;
                }
                if metric < min_activity_value {
                    min_activity_value = metric;
                }
                sum_activity_values = sum_activity_values + metric;
                filled_metrics += 1;
            }
        }
        self.metric_stats.max = max_activity_value;
        self.metric_stats.min = min_activity_value;
        self.metric_stats.sum = sum_activity_values;
        self.metric_stats.avg =
            sum_activity_values / num_traits::FromPrimitive::from_usize(filled_metrics).unwrap();
    }

    /// `get_missing_values_fill` uses the MissingValuesPolicy to decide
    /// which value to place on empty metric timeslots when drawing
    pub fn get_missing_values_fill(&mut self) -> T
    where
        T: Num + Clone + Copy + PartialOrd + Bounded + FromPrimitive,
    {
        // XXX: If the values are being shifted, these min/max will be
        // deceiving, on the other hand, it would just be deceiving for the
        // first draw after long period of inactivity, which also shows
        // visually how things are changing.
        self.calculate_stats();
        match self.missing_values_policy {
            MissingValuesPolicy::Zero => T::zero(),
            MissingValuesPolicy::One => T::one(),
            MissingValuesPolicy::Min => self.metric_stats.min,
            MissingValuesPolicy::Max => self.metric_stats.max,
            MissingValuesPolicy::Last => self.get_last_filled(),
            MissingValuesPolicy::First => self.get_first_filled(),
            MissingValuesPolicy::Avg => self.metric_stats.avg,
            MissingValuesPolicy::Fixed(val) => val,
        }
    }

    /// `resolve_metric_collision` ensures the policy for colliding values is
    /// applied.
    pub fn resolve_metric_collision(&self, existing: T, new: T) -> T {
        match self.collision_policy {
            ValueCollisionPolicy::Increment => existing + new,
            ValueCollisionPolicy::Overwrite => new,
            ValueCollisionPolicy::Decrement => existing - new,
            ValueCollisionPolicy::Ignore => existing,
        }
    }

    /// `circular_push` an item to the circular buffer
    pub fn circular_push(&mut self, input: (u64, Option<T>))
    where
        T: Num + Clone + Copy + PartialOrd + ToPrimitive + Bounded + FromPrimitive,
    {
        if self.metrics.len() < self.metrics_capacity {
            self.metrics.push(input);
            self.active_items += 1;
        } else {
            // The vector might have been invalidated because data was outdated.
            // The first and last index shorten the vector but leave old data
            // still
            if self.first_idx == 0 && self.last_idx < self.metrics_capacity {
                self.metrics[self.last_idx] = input;
            } else {
                self.metrics[self.first_idx] = input;
                self.first_idx = (self.first_idx + 1) % self.metrics_capacity;
            }
            if self.first_idx + self.last_idx < self.metrics_capacity {
                self.active_items += 1;
            }
        }
        self.last_idx = (self.last_idx + 1) % (self.metrics_capacity + 1);
    }

    /// `push` Adds values to the circular buffer adding empty entries for
    /// missing entries, may invalidate the buffer if all data is outdated
    pub fn push(&mut self, input: (u64, T))
    where
        T: Num + Clone + Copy + PartialOrd + ToPrimitive + Bounded + FromPrimitive,
    {
        if !self.metrics.is_empty() {
            let last_idx = if self.last_idx == self.metrics_capacity {
                self.metrics.len() - 1
            } else {
                self.last_idx - 1
            };
            let inactive_time = (input.0 - self.metrics[last_idx].0) as usize;
            if inactive_time > self.metrics_capacity {
                // The whole vector should be discarded
                self.first_idx = 0;
                self.last_idx = 1;
                self.metrics[0] = (input.0, Some(input.1));
                self.active_items = 1;
            } else {
                // Fill missing entries with None
                let max_epoch = self.metrics[last_idx].0;
                for fill_epoch in (max_epoch + 1)..input.0 {
                    self.circular_push((fill_epoch, None));
                }
                self.circular_push((input.0, Some(input.1)));
            }
        } else {
            self.circular_push((input.0, Some(input.1)));
        }
    }

    /// `get_last_filled` Returns the last filled entry in the circular buffer
    pub fn get_last_filled(&self) -> T
    where
        T: Clone + Copy,
    {
        let mut idx = if self.last_idx == self.metrics_capacity {
            0
        } else {
            self.last_idx - 1
        };
        loop {
            if let Some(res) = self.metrics[idx].1 {
                return res;
            }
            idx = if idx == 0 {
                self.metrics.len()
            } else {
                idx - 1
            };
            if idx == self.first_idx {
                break;
            }
        }
        T::zero()
    }

    /// `get_first_filled` Returns the first filled entry in the circular buffer
    pub fn get_first_filled(&self) -> T
    where
        T: Num + Clone + Copy,
    {
        for entry in self.iter() {
            if let Some(metric) = entry.1 {
                return metric;
            }
        }
        T::zero()
    }

    /// `as_vec` Returns the circular buffer in flat vec format
    // ....[c]
    // ..[b].[d]
    // [a].....[e]
    // ..[h].[f]
    // ....[g]
    // first_idx = "^"
    // last_idx  = "v"
    // [a][b][c][d][e][f][g][h]
    //  0  1  2  3  4  5  6  7
    //  ^v                        # empty
    //  ^  v                      # 0
    //  ^                       v # vec full
    //  v                    ^    # 7
    pub fn as_vec(&self) -> Vec<(u64, Option<T>)>
    where
        T: Clone + Copy,
    {
        if self.metrics.is_empty() {
            return vec![];
        }
        let mut res: Vec<(u64, Option<T>)> = Vec::with_capacity(self.metrics_capacity);
        for entry in self.iter() {
            res.push(entry.clone());
        }
        res
    }

    fn push_current_epoch(&mut self, input: T)
    where
        T: Num + Clone + Copy + PartialOrd + ToPrimitive + Bounded + FromPrimitive,
    {
        let now = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.push((now, input));
    }

    // `iter` Returns an Iterator from the current start.
    fn iter<'a>(&'a self) -> IterTimeSeries<'a, T>
    where
        T: Copy + Clone,
    {
        IterTimeSeries {
            inner: self,
            pos: self.first_idx,
            current_item: 0,
        }
    }
}

impl<'a, T> Iterator for IterTimeSeries<'a, T>
where
    T: Num + Clone + Copy,
{
    type Item = &'a (u64, Option<T>);
    fn next(&mut self) -> Option<Self::Item> {
        if self.inner.metrics.is_empty() || self.current_item == self.inner.active_items {
            return None;
        }
        let curr_pos = self.pos % self.inner.metrics.len();
        self.pos = (self.pos + 1) % (self.inner.metrics.len() + 1);
        self.current_item += 1;
        Some(&self.inner.metrics[curr_pos])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_pushes_circular_buffer() {
        // The circular buffer inserts rotating the first and last index
        let mut test = TimeSeries::default().with_capacity(4);
        test.circular_push((10, Some(0)));
        assert_eq!(test.first_idx, 0);
        assert_eq!(test.last_idx, 1);
        test.circular_push((11, Some(1)));
        test.circular_push((12, None));
        test.circular_push((13, Some(3)));
        assert_eq!(test.first_idx, 0);
        assert_eq!(test.last_idx, 4);
        assert_eq!(
            test.metrics,
            vec![(10, Some(0)), (11, Some(1)), (12, None), (13, Some(3))]
        );
        test.circular_push((14, Some(4)));
        assert_eq!(
            test.metrics,
            vec![(14, Some(4)), (11, Some(1)), (12, None), (13, Some(3))]
        );
        assert_eq!(test.first_idx, 1);
        assert_eq!(test.last_idx, 0);
        test.circular_push((15, Some(5)));
        assert_eq!(
            test.metrics,
            vec![(14, Some(4)), (15, Some(5)), (12, None), (13, Some(3))]
        );
        assert_eq!(test.first_idx, 2);
        assert_eq!(test.last_idx, 1);
    }
    #[test]
    fn it_gets_last_filled_value() {
        let mut test = TimeSeries::default().with_capacity(4);
        // Some values should be inserted as None
        test.push((10, 0));
        test.circular_push((11, None));
        test.circular_push((12, None));
        test.circular_push((13, None));
        assert_eq!(test.get_last_filled(), 0);
        let mut test = TimeSeries::default().with_capacity(4);
        test.circular_push((11, None));
        test.push((12, 2));
    }
    #[test]
    fn it_transforms_to_flat_vec() {
        let mut test = TimeSeries::default().with_capacity(4);
        // Some values should be inserted as None
        assert_eq!(test.first_idx, 0);
        assert_eq!(test.last_idx, 0);
        test.push((10, 0));
        assert_eq!(test.first_idx, 0);
        assert_eq!(test.last_idx, 1);
        test.push((13, 3));
        assert_eq!(test.first_idx, 0);
        assert_eq!(test.last_idx, 4);
        assert_eq!(
            test.as_vec(),
            vec![(10, Some(0)), (11, None), (12, None), (13, Some(3))]
        );
        test.push((14, 4));
        // Starting at 11
        test.first_idx = 1;
        test.last_idx = 1;
        assert_eq!(
            test.as_vec(),
            vec![(11, None), (12, None), (13, Some(3)), (14, Some(4))]
        );
        // Only 11
        test.active_items = 1;
        test.first_idx = 1;
        test.last_idx = 2;
        assert_eq!(test.as_vec(), vec![(11, None)]);
        // Only 13
        test.first_idx = 3;
        test.last_idx = 4;
        test.active_items = 1;
        assert_eq!(test.as_vec(), vec![(13, Some(3))]);
        // 13, 14
        test.first_idx = 3;
        test.last_idx = 1;
        test.active_items = 2;
        assert_eq!(test.as_vec(), vec![(13, Some(3)), (14, Some(4))]);
    }
    #[test]
    fn it_fills_empty_epochs() {
        let mut test = TimeSeries::default().with_capacity(4);
        // Some values should be inserted as None
        test.push((10, 0));
        test.push((13, 3));
        assert_eq!(
            test.metrics,
            vec![(10, Some(0)), (11, None), (12, None), (13, Some(3))]
        );
        assert_eq!(test.active_items, 4);
        // Test the whole vector is discarded
        test.push((18, 8));
        assert_eq!(test.active_items, 1);
        assert_eq!(
            test.metrics,
            vec![(18, Some(8)), (11, None), (12, None), (13, Some(3))]
        );
        assert_eq!(test.first_idx, 0);
        assert_eq!(test.last_idx, 1);
        assert_eq!(test.active_items, 1);
        assert_eq!(test.as_vec(), vec![(18, Some(8))]);
        test.push((20, 0));
        assert_eq!(
            test.metrics,
            vec![(18, Some(8)), (19, None), (20, Some(0)), (13, Some(3))]
        );
        assert_eq!(test.first_idx, 0);
        assert_eq!(test.last_idx, 3);
        assert_eq!(test.active_items, 3);
        assert_eq!(
            test.as_vec(),
            vec![(18, Some(8)), (19, None), (20, Some(0))]
        );
        test.push((50, 5));
        assert_eq!(
            test.metrics,
            // Many outdated entries
            vec![(50, Some(5)), (19, None), (20, Some(0)), (13, Some(3))]
        );
        assert_eq!(test.as_vec(), vec![(50, Some(5))]);
        test.push((53, 3));
        assert_eq!(
            test.metrics,
            vec![(50, Some(5)), (51, None), (52, None), (53, Some(3))]
        );
    }
    #[test]
    fn it_applies_missing_policies() {
        let mut test_zero = TimeSeries::default().with_capacity(5);
        let mut test_one = TimeSeries::default()
            .with_capacity(5)
            .with_missing_values_policy("one".to_string());
        let mut test_min = TimeSeries::default()
            .with_capacity(5)
            .with_missing_values_policy("min".to_string());
        let mut test_max = TimeSeries::default()
            .with_capacity(5)
            .with_missing_values_policy("max".to_string());
        let mut test_last = TimeSeries::default()
            .with_capacity(5)
            .with_missing_values_policy("last".to_string());
        let mut test_first = TimeSeries::default()
            .with_capacity(5)
            .with_missing_values_policy("first".to_string());
        let mut test_avg = TimeSeries::default()
            .with_capacity(5)
            .with_missing_values_policy("avg".to_string());
        test_zero.push((0, 9));
        test_zero.push((2, 1));
        test_one.push((0, 9));
        test_one.push((2, 1));
        test_min.push((0, 9));
        test_min.push((2, 1));
        test_max.push((0, 9));
        test_max.push((2, 1));
        test_last.push((0, 9));
        test_last.push((2, 1));
        test_first.push((0, 9));
        test_first.push((2, 1));
        test_avg.push((0, 9));
        test_avg.push((2, 1));
        assert_eq!(test_zero.get_missing_values_fill(), 0);
        assert_eq!(test_one.get_missing_values_fill(), 1);
        assert_eq!(test_min.get_missing_values_fill(), 1);
        assert_eq!(test_max.get_missing_values_fill(), 9);
        assert_eq!(test_last.get_missing_values_fill(), 1);
        assert_eq!(test_first.get_missing_values_fill(), 9);
        assert_eq!(test_avg.get_missing_values_fill(), 5);
        // TODO: add Fixed value test
    }
    #[test]
    fn it_iterates_trait() {
        // Iterator Trait
        // Test an empty TimeSeries vec
        let test0: TimeSeries<i8> = TimeSeries::default().with_capacity(4);
        let mut iter_test0 = test0.iter();
        assert_eq!(iter_test0.pos, 0);
        assert!(iter_test0.next().is_none());
        assert!(iter_test0.next().is_none());
        assert_eq!(iter_test0.pos, 0);
        // Simple test with one item
        let mut test1 = TimeSeries::default().with_capacity(4);
        test1.circular_push((10, Some(0)));
        let mut iter_test1 = test1.iter();
        assert_eq!(iter_test1.next(), Some(&(10, Some(0))));
        assert_eq!(iter_test1.pos, 1);
        assert!(iter_test1.next().is_none());
        assert!(iter_test1.next().is_none());
        assert_eq!(iter_test1.pos, 1);
        // Simple test with 3 items, rotated to start first item and 2nd
        // position and last item at 3rd position
        let mut test2 = TimeSeries::default().with_capacity(4);
        test2.circular_push((10, Some(0)));
        test2.circular_push((11, Some(1)));
        test2.circular_push((12, Some(2)));
        test2.circular_push((13, Some(3)));
        test2.first_idx = 1;
        test2.last_idx = 3;
        assert_eq!(
            test2.metrics,
            vec![(10, Some(0)), (11, Some(1)), (12, Some(2)), (13, Some(3))]
        );
        let mut iter_test2 = test2.iter();
        assert_eq!(iter_test2.pos, 1);
        assert_eq!(iter_test2.next(), Some(&(11, Some(1))));
        assert_eq!(iter_test2.next(), Some(&(12, Some(2))));
        assert_eq!(iter_test2.pos, 3);
        // A vec that is completely full
        let mut test3 = TimeSeries::default().with_capacity(4);
        test3.circular_push((10, Some(0)));
        test3.circular_push((11, Some(1)));
        test3.circular_push((12, Some(2)));
        test3.circular_push((13, Some(3)));
        {
            let mut iter_test3 = test3.iter();
            assert_eq!(iter_test3.next(), Some(&(10, Some(0))));
            assert_eq!(iter_test3.next(), Some(&(11, Some(1))));
            assert_eq!(iter_test3.next(), Some(&(12, Some(2))));
            assert_eq!(iter_test3.next(), Some(&(13, Some(3))));
            assert!(iter_test3.next().is_none());
            assert!(iter_test3.next().is_none());
            assert_eq!(iter_test2.pos, 3);
        }
        // After changing the data the idx is recreatehd at 11 as expected
        test3.circular_push((14, Some(4)));
        let mut iter_test3 = test3.iter();
        assert_eq!(iter_test3.next(), Some(&(11, Some(1))));
    }
    // let size = SizeInfo{
    // width: 100f32,
    // height: 100f32,
    // cell_width: 1f32,
    // cell_height: 1f32,
    // padding_x: 0f32,
    // padding_y: 0f32,
    // dpr: 1f64
    // };
}
// `draw` sends the time series representation of the TimeSeries to OpenGL
// context, this shouldn't be mut
// fn draw(&self);
// fn update_opengl_vecs(size: SizeInfo) -> Vec<f32>{
// unimplemented!("XXX");
// }
// Every time unit (currently second) is stored as an item in the array
// Rotation may be needed due to inactivity or the array being filled
// self.rotate_activity_levels_vec(input.0);
// activity_time_length = metrics.len();
// if activity_time_length < self.max_activity_ticks {
// self.activity_levels.push(new_value);
// } else {
// self.activity_levels[activity_time_length - 1] = new_value;
// }
// TODO: self.update_activity_opengl_vecs(size);
// }
// fn from_prometheus(&mut self, &mut metrics: Vec<(u64, Self::MetricType)>, url: String)
//     where Self::MetricType: Num + Clone + Copy + PartialOrd + ToPrimitive + Bounded +
// FromPrimitive {
// }
//
// `init_opengl_context` provides a default initialization of OpengL
// context. This function is called previous to sending the vector data.
// This seems to be part of src/renderer/ mod tho...
// fn init_opengl_context(&self);
// }
//
// `ActivityLevels` keep track of the activity per time tick
// Currently this is a second as we use UNIX_EPOCH
// #[derive(Debug, Clone)]
// pub struct ActivityLevels<T>
// where T: Num + Clone + Copy
// {
// Capture events/characters per second
// Contains one entry per second
// pub activity_levels: Vec<T>,
//
// Last Activity Time
// pub last_activity_time: u64,
//
// Max activity ticks to show, ties to the activity_levels array
// it should cause it to throw away old items for newer records
// pub max_activity_ticks: usize,
//
// The color of the activity_line
// pub color: Rgb,
//
// The offset in which the activity line should be drawn
// pub x_offset: f32,
//
// The width of the activity chart/histogram
// pub width: f32,
//
// The opengl representation of the activity levels
// Contains twice as many items because it's x,y
// pub activity_opengl_vecs: Vec<f32>,
//
// The height of the activity line region
// pub activity_line_height: f32,
//
// The spacing between the activity line segments, could be renamed to line length
// pub activity_tick_spacing: f32,
//
// The transparency of the activity line
// pub alpha: f32,
//
// Useful for records that do not increment but rather are a fixed
// or absolute value recorded at a given time
// pub overwrite_last_entry: bool,
//
// A marker line to indicate a reference point, for example for load
// to show where the 1 loadavg is, or to show disk capacity
// pub marker_line: Option<T>,
//
// The opengl representation of the activity levels
// Contains twice as many items because it's x,y
// pub marker_line_vecs: Vec<f32>,
//
// Missing values can be set to zero
// to show where the 1 task per core is
// pub missing_values_policy: MissingValuesPolicy<T>,
// }
//
// impl<T> Default for ActivityLevels<T>
// where T: Num + Clone + Copy
// {
// `default` provides 5mins activity lines with red color
// The vector of activity_levels per second is created with reserved capacity,
// just to avoid needed to reallocate the vector in memory the first 5mins.
// fn default() -> ActivityLevels<T>{
// let activity_time = std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
// let activity_vector_capacity = 300usize; // 300 seconds (5mins)
// ActivityLevels{
// last_activity_time: activity_time,
// activity_levels: Vec::<T>::with_capacity(activity_vector_capacity), // XXX: Maybe use vec![0;
// 300]; to pre-fill with zeros? max_activity_ticks: activity_vector_capacity,
// color: Rgb { // By default red
// r: 183,
// g: 28,
// b: 28,
// },
// x_offset: 600f32,
// width: 200f32,
// activity_opengl_vecs: Vec::<f32>::with_capacity(activity_vector_capacity * 2),
// marker_line_vecs: vec![0f32; 16],
// activity_line_height: 25f32,
// activity_tick_spacing: 5f32,
// alpha: 1f32,
// overwrite_last_entry: false,
// marker_line: None,
// missing_values_policy: MissingValuesPolicy::Zero,
// }
// }
// }
//
// impl<T> ActivityLevels<T>
// where T: Num + Clone + Copy
// {
// `with_color` Changes the color of the activity line
// pub fn with_color(mut self, color: Rgb) -> ActivityLevels<T> {
// self.color = color;
// self
// }
//
// `with_x_offset` Changes the offset of the activity level drawing location
// pub fn with_x_offset(mut self, new_offset: f32) -> ActivityLevels<T> {
// self.x_offset = new_offset;
// self
// }
//
// `with_width` Changes the width of the activity level drawing location
// pub fn with_width(mut self, width: f32) -> ActivityLevels<T> {
// self.width = width;
// self
// }
//
// `with_alpha` Changes the transparency of the activity level
// pub fn with_alpha(mut self, alpha: f32) -> ActivityLevels<T> {
// self.alpha = alpha;
// self
// }
//
// `with_overwrite_last_entry` overwrite instead of increment current time
// entry
// pub fn with_overwrite_last_entry(mut self, value: bool) -> ActivityLevels<T> {
// self.overwrite_last_entry = value;
// self
// }
//
// `with_marker_line` initializes the marker line into a Some
// pub fn with_marker_line(mut self, value: T) -> ActivityLevels<T> {
// self.marker_line = Some(value);
// self
// }
// `update_activity_level` Ensures time slots are filled with 0s for
// inactivity and increments the current epoch activity_level slot by an
// new_value, it uses the size to calculate the position from the
// bottom in which to display the activity levels
// pub fn update_activity_level(&mut self,
// size: SizeInfo,
// new_value: T
// )
// where T: Num + Clone + Copy + PartialOrd + ToPrimitive + Bounded + FromPrimitive
// {
// XXX: Right now set to "as_secs", but could be used for other time units easily
// let mut activity_time_length = self.activity_levels.len();
// let now = std::time::SystemTime::now()
// .duration_since(UNIX_EPOCH)
// .unwrap()
// .as_secs();
// if activity_time_length == 0 {
// The vector is empty, no need to rotate or do anything special
// self.activity_levels.push(new_value);
// self.last_activity_time = now;
// self.update_activity_opengl_vecs(size);
// return;
// }
// if now == self.last_activity_time {
// The Vector is populated and has one active item at least which
// we can work on, no need to rotate or do anything special
// if self.overwrite_last_entry {
// self.activity_levels[activity_time_length - 1] = new_value;
// } else {
// self.activity_levels[activity_time_length - 1] = self.activity_levels[activity_time_length - 1] +
// new_value; }
// self.update_activity_opengl_vecs(size);
// return;
// }
// Every time unit (currently second) is stored as an item in the array
// Rotation may be needed due to inactivity or the array being filled
// self.rotate_activity_levels_vec(now);
// activity_time_length = self.activity_levels.len();
// if activity_time_length < self.max_activity_ticks {
// self.activity_levels.push(new_value);
// } else {
// self.activity_levels[activity_time_length - 1] = new_value;
// }
// self.last_activity_time = now;
// self.update_activity_opengl_vecs(size);
// }
//
// `scale_y_to_size` Scales the value to the current display boundary
// pub fn scale_y_to_size(&self, size: SizeInfo, input_value: T, max_activity_value: T) -> f32
// where T: Num + ToPrimitive
// {
// let center_y = size.height / 2.;
// let y = size.height -
// 2. * size.padding_y -
// ( self.activity_line_height *
// num_traits::ToPrimitive::to_f32(&input_value).unwrap() /
// num_traits::ToPrimitive::to_f32(&max_activity_value).unwrap()
// );
// -(y - center_y) / center_y
// }
//
// `scale_x_to_size` Scales the value to the current display boundary
// pub fn scale_x_to_size(&self, size: SizeInfo, input_value: f32) -> f32
// where T: Num + ToPrimitive
// {
// let center_x = size.width / 2.;
// let x = size.padding_x + self.x_offset + input_value;
// (x - center_x) / center_x
// }
//
//
// `update_marker_line_vecs` Scales the Marker Line to the current size of
// the displayed points
// pub fn update_marker_line_vecs(&mut self, size: SizeInfo, max_activity_value: T,
// marker_line_position: T) where T: Num + PartialOrd + ToPrimitive + Bounded + FromPrimitive
// {
// TODO: Add marker_line color
// Draw a marker at a fixed position for reference: |>---------<|
// The vertexes of the above marker idea can be represented as
// connecting lines for these coordinates:
// x2,y2                               x6,y2
//       x1,y1                   x5,y1
// x2,y3                               x6,y3
// |-   10%   -|-     80%     -|-   10%   -|
//
// Calculate X, the triangle width is 10% of the available draw space
// let x1 = self.scale_x_to_size(size, self.width / 10f32);
// let x2 = self.scale_x_to_size(size, 0f32);
// let x5 = self.scale_x_to_size(size, self.width - self.width / 10f32);
// let x6 = self.scale_x_to_size(size, self.width);
//
// Calculate X, the triangle height is 10% of the available draw space
// let y1 = self.scale_y_to_size(size,
// marker_line_position,
// max_activity_value); // = y4,y5,y8
// let y2 = y1 - self.scale_y_to_size(size,max_activity_value,max_activity_value) / 100f32; // = y7
// let y3 = y1 + self.scale_y_to_size(size,max_activity_value,max_activity_value) / 100f32; // = y7
//
// Left triangle |>
// self.marker_line_vecs[0] = x1;
// self.marker_line_vecs[1] = y1;
// self.marker_line_vecs[2] = x2;
// self.marker_line_vecs[3] = y2;
// self.marker_line_vecs[4] = x2;
// self.marker_line_vecs[5] = y3;
//
// Line from left triangle to right triangle ---
// self.marker_line_vecs[6] = x1;
// self.marker_line_vecs[7] = y1;
// self.marker_line_vecs[8] = x5;
// self.marker_line_vecs[9] = y1;
//
// Right triangle <|
// self.marker_line_vecs[10] = x6;
// self.marker_line_vecs[11] = y3;
// self.marker_line_vecs[12] = x6;
// self.marker_line_vecs[13] = y2;
//
// And loop back to x5,y5
// self.marker_line_vecs[14] = x5;
// self.marker_line_vecs[15] = y1;
//
// }
//
// `update_opengl_vecs` Represents the activity levels values in a
// drawable vector for opengl
// pub fn update_activity_opengl_vecs(&mut self, size: SizeInfo)
// where T: Num + PartialOrd + ToPrimitive + Bounded + FromPrimitive
// {
// Get the maximum value
// let mut max_activity_value = T::zero();
// for idx in 0..self.activity_levels.len() {
// if self.activity_levels[idx] > max_activity_value {
// max_activity_value = self.activity_levels[idx];
// }
// }
// if let Some(marker_line_value) = self.marker_line {
// if marker_line_value > max_activity_value {
// max_activity_value = marker_line_value;
// }
// }
// Get the opengl representation of the vector
// let opengl_vecs_len = self.activity_opengl_vecs.len();
// Calculate the tick spacing
// let tick_spacing = if self.marker_line.is_some() {
// Subtract 20% of the horizonal draw space that is allocated for
// the Marker Line
// self.width * 0.2 / self.max_activity_ticks as f32
// } else {
// self.width / self.max_activity_ticks as f32
// };
// for idx in 0..self.activity_levels.len() {
// let mut x_value = idx as f32 * tick_spacing;
// If there is a Marker Line, it takes 10% of the initial horizontal space
// if self.marker_line.is_some() {
// x_value += self.width * 0.1;
// }
// let scaled_x = self.scale_x_to_size(size, x_value);
// let scaled_y = self.scale_y_to_size(size, self.activity_levels[idx], max_activity_value);
// Adding twice to a vec, could this be made into one operation? Is this slow?
// need to transform activity line values from varying levels into scaled [-1, 1]
// if (idx + 1) * 2 > opengl_vecs_len {
// self.activity_opengl_vecs.push(scaled_x);
// self.activity_opengl_vecs.push(scaled_y);
// } else {
// self.activity_opengl_vecs[idx * 2] = scaled_x;
// self.activity_opengl_vecs[idx * 2 + 1] = scaled_y;
// }
// }
// if let Some(marker_line_value) = self.marker_line {
// self.update_marker_line_vecs(size, max_activity_value, marker_line_value);
// }
// }
// }
//
// pub struct PrometheusMetric {
// name: String,
// url: String,
// }
//
// impl PrometheusMetric {
// pub fn load() -> Future {
// rt::run(rt::lazy(|| {
// let client = Client::new();
// let uri = format!("http://localhost:9090/api/v1/query?{}",query)
// .parse()
// .unwrap();
// client
// .get(uri)
// .and_then(|res| {
// println!("Response: {}", res.status());
// res
// .into_body()
// Body is a stream, so as each chunk arrives...
// .for_each(|chunk| {
// io::stdout()
// .write_all(&chunk)
// .map_err(|e| {
// panic!("example expects stdout is open, error={}", e)
// })
// })
// })
// .map_err(|err| {
// println!("Error: {}", err);
// })
// }));
// }
// pub fn new(name: String, query: String) -> PrometheusMetric {
// PrometheusMetric {
// url: format!("http://localhost:9090/api/v1/query?{}",query),
// name,
// }
// }
// }
//
// pub struct LoadAvg {
// From https://docs.rs/procinfo/0.4.2/procinfo/struct.LoadAvg.html
// Load average over the last minute.
// load_avg_1_min: ActivityLevels<f32>,
//
// Load average of the last 5 minutes.
// load_avg_5_min: ActivityLevels<f32>,
//
// Load average of the last 10 minutes
// load_avg_10_min: ActivityLevels<f32>,
//
// These metrics are not that interesting to graph:
// the number of currently runnable kernel scheduling entities (processes, threads).
// tasks_runnable: ActivityLevels<u32>,
// the number of kernel scheduling entities that currently exist on the system.
// tasks_total: ActivityLevels<u32>,
//
// If no metrics were collected for some time, fill them with the last
// known value
// missing_values_policy: MissingValuesPolicy<f32>,
//
// A marker line to indicate a reference point, for example for load
// to show where the 1 loadavg is, or to show disk capacity
// pub marker_line: Option<f32>,
//
// The opengl representation of the activity levels
// Contains twice as many items because it's x,y
// pub marker_line_vecs: Vec<f32>,
// }
//
// impl Default for LoadAvg{
// fn default() -> LoadAvg {
// LoadAvg{
// load_avg_1_min: ActivityLevels::default()
// .with_color(Rgb{r:93,g:23,b:106})
// .with_width(50f32)
// .with_alpha(0.9)
// .with_missing_values_policy("last".to_string())
// .with_marker_line(1f32)
// .with_overwrite_last_entry(true)
// .with_x_offset(1010f32),
// load_avg_5_min: ActivityLevels::default()
// .with_color(Rgb{r:146,g:75,b:158})
// .with_width(30f32)
// .with_alpha(0.6)
// .with_missing_values_policy("last".to_string())
// .with_marker_line(1f32)
// .with_overwrite_last_entry(true)
// .with_x_offset(1070f32),
// load_avg_10_min: ActivityLevels::default()
// .with_color(Rgb{r:202,g:127,b:213})
// .with_width(20f32)
// .with_alpha(0.3)
// .with_missing_values_policy("last".to_string())
// .with_marker_line(1f32) // Set a reference point at load 1
// .with_overwrite_last_entry(true)
// .with_x_offset(1110f32),
// tasks_runnable: ActivityLevels::default()
//    .with_color(Rgb{r:0,g:172,b:193})
//    .with_width(50f32)
//    .with_missing_values_policy("last".to_string())
//    .with_overwrite_last_entry(true)
//    .with_x_offset(1140f32),
// tasks_total: ActivityLevels::default()
//    .with_color(Rgb{r:27,g:160,b:71})
//    .with_width(50f32)
//    .with_missing_values_policy("last".to_string())
//    .with_overwrite_last_entry(true)
//    .with_x_offset(1190f32),
// missing_values_policy: MissingValuesPolicy::Last,
// marker_line: Some(1f32),
// marker_line_vecs: vec![0f32; 16],
// }
// }
// }
//
// impl TimeSeries for LoadAvg {
// type MetricType = f32;
// fn draw(&self) {
// }
// fn max(&self, _input: &Vec<Self::MetricType>) -> Self::MetricType
// where Self::MetricType: Num + PartialOrd
// {
// let mut max_activity_value = TimeSeries::max(self, &self.load_avg_1_min.activity_levels);
// let max_5_min = TimeSeries::max(self, &self.load_avg_5_min.activity_levels);
// if max_activity_value < max_5_min {
// max_activity_value = max_5_min;
// }
// let max_10_min = TimeSeries::max(self, &self.load_avg_10_min.activity_levels);
// if max_activity_value < max_10_min {
// max_activity_value = max_10_min;
// }
// max_activity_value
// }
//
// }

// impl TimeSeries for ActivityLevels {
//    fn draw()
//}
