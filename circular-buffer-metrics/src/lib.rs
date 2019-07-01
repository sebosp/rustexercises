//! Exports the TimeSeries class
//! The TimeSeries is a circular buffer that contains an entry per epoch
//! at different granularities. It is maintained as a Vec<(u64, T)> where
//! T is a metric. Since metrics will overwrite the contents of the array
//! partially, the start of the metrics and the end of the metrics are
//! maintained as two separate indexes. This allows the vector to shrink
//! and rotate without relocation of memory or shifting of the vector.

// DONE:
// -- Add step to query (1 second resolution for example)
// -- Add min/max time to query.
// -- Move to config.yaml
// -- The yaml should drive an array of activity dashboards
// IN PROGRESS:
// -- Use prometheus queries instead of our own aggregation/etc.
// -- Group labels into separate colors (find something that does color spacing in rust)
// -- Logging
// TODO:
// -- The dashboards should be toggable, some key combination
// -- When activated on toggle it could blur a portion of the screen
// -- derive builder
// -- mock the prometheus server and response
// -- Tokio timers

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

extern crate futures;
extern crate hyper;
extern crate percent_encoding;
extern crate serde;
extern crate serde_json;
extern crate tokio;
extern crate tokio_core;
// use crate::term::color::Rgb;
// use crate::term::SizeInfo;
use log::*;
use std::time::UNIX_EPOCH;

pub mod config;
pub mod prometheus;

/// `MissingValuesPolicy` provides several ways to deal with missing values
/// when drawing the Metric
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum MissingValuesPolicy {
    Zero,
    One,
    First,
    Last,
    Fixed(f64),
    Avg,
    Max,
    Min,
}

impl Default for MissingValuesPolicy {
    fn default() -> MissingValuesPolicy {
        MissingValuesPolicy::Zero
    }
}

/// `ValueCollisionPolicy` handles collisions when several values are collected
/// for the same time unit, allowing for overwriting, incrementing, etc.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TimeSeriesStats {
    max: f64,
    min: f64,
    avg: f64, // Calculation may lead to overflow
    first: f64,
    last: f64,
    count: usize,
    sum: f64, // May overflow
    is_dirty: bool,
}

impl Default for TimeSeriesStats {
    fn default() -> TimeSeriesStats {
        TimeSeriesStats {
            max: 0f64,
            min: 0f64,
            avg: 0f64,
            first: 0f64,
            last: 0f64,
            count: 0usize,
            sum: 0f64,
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
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TimeSeries {
    /// Capture events through time
    /// Contains one entry per time unit
    pub metrics: Vec<(u64, Option<f64>)>,

    /// Number of items to store in our metrics vec
    pub metrics_capacity: usize,

    /// Stats for the TimeSeries
    pub stats: TimeSeriesStats,

    /// Useful for records that do not increment but rather are a fixed
    /// or absolute value recorded at a given time
    pub collision_policy: ValueCollisionPolicy,

    /// Missing values can be set to zero
    /// to show where the 1 task per core is
    pub missing_values_policy: MissingValuesPolicy,

    /// The first item in the circular buffer
    pub first_idx: usize,

    /// The last item in the circular buffer
    pub last_idx: usize,

    /// The circular buffer has two indexes, if the start and end
    /// indexes are the same, then the buffer is full or has one item
    /// By knowing the active_items in advance we know which situation is true
    pub active_items: usize,
}

/// `IterTimeSeries` provides the Iterator Trait for TimeSeries metrics.
/// The state for the iteration is held en "pos" field. The "current_item" is
/// used to determine if further iterations on the circular buffer is needed.
pub struct IterTimeSeries<'a> {
    /// The reference to the TimeSeries struct to iterate over.
    inner: &'a TimeSeries,
    /// The current position state
    pos: usize,
    /// The current item number, to be compared with the active_items
    current_item: usize,
}

/// `ReferencePointDecoration` draws a fixed point to give a reference point
/// of what a drawn value may mean
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ReferencePointDecoration {
    /// The value at which to draw the reference point
    pub value: f64,

    /// The reference point will use additional height for the axis line
    /// this makes it fit in the configured space, basically the value
    /// will be incremented by this additional percentage to give more
    /// space to draw the axis tick
    #[serde(default)]
    pub height_multiplier: f64,

    /// hexadecimal color
    #[serde(default)]
    pub color: String,

    /// Transparency
    #[serde(default)]
    pub alpha: f32,

    /// The pixels to separate from the left and right
    #[serde(default)]
    pub padding: Value2D,

    /// The opengl vertices is stored in this vector
    /// The capacity is always 12, see opengl_vertices()
    #[serde(default)]
    pub opengl_data: Vec<f32>,
}

impl Default for ReferencePointDecoration {
    fn default() -> ReferencePointDecoration {
        ReferencePointDecoration {
            value: 1.0,
            height_multiplier: 0.05,
            color: String::from("0xff0000"),
            alpha: 1.0,
            padding: Value2D {
                x: 10f32,
                y: 0f32, // No top/bottom padding
            },
            opengl_data: vec![0.; 12],
        }
    }
}

impl ReferencePointDecoration {
    /// `opengl_vertices` Scales the Marker Line to the current size of
    /// the displayed points
    pub fn opengl_vertices(&self) -> Vec<f32> {
        self.opengl_data.clone()
    }

    /// `top_value` increments the reference point value by an additional
    /// percentage to account for space to draw the axis tick
    pub fn top_value(&self) -> f64 {
        self.value + self.value * self.height_multiplier
    }

    /// `bottom_value` decrements the reference point value by a percentage
    /// to account for space to draw the axis tick
    pub fn bottom_value(&self) -> f64 {
        self.value - self.value * self.height_multiplier
    }

    /// `update_opengl_vertices` Draws a marker at a fixed position for
    /// reference.
    pub fn update_opengl_vertices(
        &mut self,
        display_size: SizeInfo,
        offset: Value2D,
        chart_width: f32,
        chart_max_value: f64,
    ) {
        debug!("Starting update_opengl_vertices");
        // The vertexes of the above marker idea can be represented as
        // connecting lines for these coordinates:
        //         |Actual Draw Metric Data|
        // x1,y2   |                       |   x2,y2
        // x1,y1 --|-----------------------|-- x2,y1
        // x1,y3   |                       |   x2,y3
        // |- 10% -|-         80%         -|- 10% -|
        // TODO: Add marker_line color to opengl
        // TODO: Call only when max or min have changed in collected metrics
        //
        // Calculate X coordinates:
        let x1 = display_size.scale_x(offset.x);
        let x2 = display_size.scale_x(offset.x + chart_width);

        // Calculate Y, the marker hints are 10% of the current values
        // This means that the
        let y1 = display_size.scale_y(chart_max_value, self.value);
        let y2 = display_size.scale_y(chart_max_value, self.top_value());
        let y3 = display_size.scale_y(chart_max_value, self.bottom_value());

        // Build the left most axis "tick" mark.
        self.opengl_data[0] = x1;
        self.opengl_data[1] = y2;
        self.opengl_data[2] = x1;
        self.opengl_data[3] = y3;

        // Create the line to the other side
        self.opengl_data[4] = x1;
        self.opengl_data[5] = y1;
        self.opengl_data[6] = x2;
        self.opengl_data[7] = y1;
        //
        // Finish the axis "tick" on the other side
        self.opengl_data[8] = x2;
        self.opengl_data[9] = y3;
        self.opengl_data[10] = x2;
        self.opengl_data[11] = y2;
        debug!("Finished update_opengl_vertices: {:?}", self.opengl_data);
    }
}

/// `Decoration` contains several types of decorations to add to a chart
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(tag = "type")]
pub enum Decoration {
    #[serde(rename = "reference")]
    Reference(ReferencePointDecoration),
    None,
    // Maybe add Average, threshold coloring (turn line red after a certain
    // point)
}

impl Default for Decoration {
    fn default() -> Decoration {
        Decoration::None
    }
}

impl Decoration {
    /// `width` of the Decoration as it may need space to be drawn, otherwise
    /// the decoration and the data itself would overlap, these are pixels
    fn width(&self) -> f32 {
        match self {
            Decoration::Reference(d) => d.padding.x,
            Decoration::None => 0f32,
        }
    }

    /// `top_value` is the Y value of the decoration, it needs to be
    /// in the range of the metrics that have been collected, thus f64
    /// this is the highest point the Decoration will use
    fn top_value(&self) -> f64 {
        match self {
            Decoration::Reference(ref d) => d.top_value(),
            Decoration::None => 0f64,
        }
    }

    /// `bottom_value` is the Y value of the decoration, it needs to be
    /// in the range of the metrics that have been collected, thus f64
    /// this is the lowest point the Decoration will use
    fn bottom_value(&self) -> f64 {
        match self {
            Decoration::Reference(d) => d.value - d.value * d.height_multiplier,
            Decoration::None => 0f64,
        }
    }

    /// `opengl_vertices` is the representation of the decoration in opengl
    /// These are for now GL_LINES and 2D
    fn update_opengl_vertices(
        &mut self,
        display_size: SizeInfo,
        offset: Value2D,
        chart_width: f32,
        chart_max_value: f64,
    ) {
        match self {
            Decoration::Reference(ref mut d) => {
                d.update_opengl_vertices(display_size, offset, chart_width, chart_max_value)
            }
            Decoration::None => (),
        }
    }
    /// `opengl_vertices` returns the representation of the decoration in
    /// opengl. These are for now GL_LINES and 2D
    fn opengl_vertices(&mut self) -> Vec<f32> {
        match self {
            Decoration::Reference(d) => d.opengl_vertices(),
            Decoration::None => vec![],
        }
    }
}

/// `ManualTimeSeries` is a 2D struct from top left being 0,0
/// and bottom right being display limits in pixels
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ManualTimeSeries {
    /// The name of the ManualTimeSeries
    pub name: String,

    /// The TimeSeries that contains the data
    #[serde(default)]
    pub series: TimeSeries,

    /// The capacity (amount of entries to store)
    #[serde(default)]
    pub capacity: usize,

    /// The granularity to store
    #[serde(default)]
    pub granularity: u64,
}

impl Default for ManualTimeSeries {
    fn default() -> ManualTimeSeries {
        ManualTimeSeries {
            name: String::from("unkown"),
            series: TimeSeries::default(),
            capacity: 300usize, // 5 minutes
            granularity: 1,     // 1 second
        }
    }
}

/// `TimeSeriesSource` contains several types of time series that can be extended
/// with drawable data
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(tag = "type")]
pub enum TimeSeriesSource {
    #[serde(rename = "prometheus")]
    PrometheusTimeSeries(prometheus::PrometheusTimeSeries),
    #[serde(rename = "alacritty_input")]
    AlacrittyInput(ManualTimeSeries),
    #[serde(rename = "alacritty_output")]
    AlacrittyOutput(ManualTimeSeries),
}

impl Default for TimeSeriesSource {
    fn default() -> TimeSeriesSource {
        TimeSeriesSource::AlacrittyInput(ManualTimeSeries::default())
    }
}

impl TimeSeriesSource {
    fn series(&self) -> TimeSeries {
        match self {
            TimeSeriesSource::PrometheusTimeSeries(x) => x.series.clone(),
            TimeSeriesSource::AlacrittyInput(x) => x.series.clone(),
            TimeSeriesSource::AlacrittyOutput(x) => x.series.clone(),
        }
    }
    fn series_mut(&mut self) -> &mut TimeSeries {
        match self {
            TimeSeriesSource::PrometheusTimeSeries(x) => &mut x.series,
            TimeSeriesSource::AlacrittyInput(x) => &mut x.series,
            TimeSeriesSource::AlacrittyOutput(x) => &mut x.series,
        }
    }
    pub fn name(&self) -> String {
        match self {
            TimeSeriesSource::PrometheusTimeSeries(x) => x.name.clone(),
            TimeSeriesSource::AlacrittyInput(x) => x.name.clone(),
            TimeSeriesSource::AlacrittyOutput(x) => x.name.clone(),
        }
    }
}

/// `Value2D` provides X,Y values for several uses, such as offset, padding
#[derive(Default, Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub struct Value2D {
    #[serde(default)]
    x: f32,
    #[serde(default)]
    y: f32,
}

/// `SizeInfo` is a copy of the Alacritty SizeInfo, XXX: remove on merge.
#[derive(Default, Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub struct SizeInfo {
    width: f32,
    height: f32,
    cell_width: f32,
    cell_height: f32,
    padding_x: f32,
    padding_y: f32,
}

impl SizeInfo {
    /// `scale_x` Scales the value from the current display boundary to
    /// a cartesian plane from [-1.0, 1.0], where -1.0 is 0px (left-most) and
    /// 1.0 is the `display_width` parameter (right-most), i.e. 1024px.
    pub fn scale_x(&self, input_value: f32) -> f32 {
        let center_x = self.width / 2.;
        let x = self.padding_x + input_value;
        (x - center_x) / center_x
    }

    /// `scale_y_to_size` Scales the value from the current display boundary to
    /// a cartesian plane from [-1.0, 1.0], where 1.0 is 0px (top) and -1.0 is
    /// the `display_height` parameter (bottom), i.e. 768px.
    pub fn scale_y(&self, max_value: f64, input_value: f64) -> f32 {
        let center_y = self.height / 2.;
        let y = self.height
            - 2. * self.padding_y
            - (self.height * (input_value as f32) / max_value as f32);
        -(y - center_y) / center_y
    }
}

/// `TimeSeriesChart` has an array of TimeSeries to display, it contains the
/// X, Y position and has methods to draw in opengl.
#[derive(Default, Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TimeSeriesChart {
    /// The name of the Chart
    pub name: String,

    /// The different sources of the TimeSeries to draw
    #[serde(rename = "series")]
    pub sources: Vec<TimeSeriesSource>,

    /// Decorations such as color, transparency, etc
    #[serde(default)]
    pub decorations: Vec<Decoration>,

    /// The merged stats of the TimeSeries
    #[serde(default)]
    pub stats: TimeSeriesStats,

    /// The offset in which the activity line should be drawn
    #[serde(default)]
    pub offset: Value2D,

    /// The width of the activity chart/histogram
    #[serde(default)]
    pub width: f32,

    /// The height of the activity line region
    #[serde(default)]
    pub height: f32,

    /// The spacing between the activity line segments, could be renamed to line length
    #[serde(default)]
    pub tick_spacing: f32,

    /// The opengl representation of the activity levels
    /// Contains twice as many items because it's x,y
    #[serde(default)]
    pub series_opengl_vecs: Vec<f32>,
}

impl TimeSeriesChart {
    /// `update_opengl_vecs` Represents the activity levels values in a
    /// drawable vector for opengl
    pub fn update_opengl_vecs(&mut self, display_size: SizeInfo) {
        debug!("Chart: Starting update_opengl_vecs");
        // Get the opengl representation of the vector
        let opengl_vecs_len = self.sources.iter().fold(0usize, |acc: usize, source| {
            acc + source.series().active_items
        });
        for source in &mut self.sources {
            if source.series().stats.is_dirty {
                debug!(
                    "Chart: {} stats are dirty, needs recalculating",
                    source.name()
                );
                source.series_mut().calculate_stats();
            }
        }
        self.calculate_stats();
        let mut decorations_space = 0f32;
        for decoration in &self.decorations {
            debug!("Chart: Adding width of decoration: {}", decoration.width());
            decorations_space += decoration.width();
        }
        for source in self.sources.iter() {
            let idx = 0usize;
            let missing_values_fill = source.series().get_missing_values_fill();
            debug!(
                "Chart: Using {} to fill missing values",
                missing_values_fill
            );
            // Calculate the tick spacing
            let tick_spacing =
                (self.width - decorations_space) / source.series().metrics_capacity as f32;
            debug!("Chart: Using tick_spacing {}", tick_spacing);
            for metric in source.series().iter() {
                // The decorations width request is on both left and right.
                let x_value = idx as f32 * tick_spacing + (decorations_space / 2f32);
                // If there is a Marker Line, it takes 10% of the initial horizontal space
                let y_value = match metric.1 {
                    Some(x) => x,
                    None => missing_values_fill,
                };
                let scaled_x = display_size.scale_x(x_value + self.offset.x);
                let scaled_y = display_size.scale_y(self.stats.max, y_value);
                // Adding twice to a vec, could this be made into one operation? Is this slow?
                // need to transform activity line values from varying levels into scaled [-1, 1]
                if (idx + 1) * 2 > opengl_vecs_len {
                    self.series_opengl_vecs.push(scaled_x);
                    self.series_opengl_vecs.push(scaled_y);
                } else {
                    // XXX: This needs fixing
                    self.series_opengl_vecs[idx * 2] = scaled_x;
                    self.series_opengl_vecs[idx * 2 + 1] = scaled_y;
                }
            }
        }
        for decoration in &mut self.decorations {
            debug!("Chart: Updating decoration {:?} vertices", decoration);
            decoration.update_opengl_vertices(
                display_size,
                self.offset,
                self.width,
                self.stats.max,
            );
        }
    }

    /// `calculate_stats` Iterates over the time series stats and merges them.
    /// This will also go through the decorations and account for the requested
    /// draw space for them.
    pub fn calculate_stats(&mut self) {
        let mut max_activity_value = std::f64::MIN;
        let mut min_activity_value = std::f64::MAX;
        let mut sum_activity_values = 0f64;
        let mut filled_stats = 0usize;
        for source in &mut self.sources {
            if source.series_mut().stats.is_dirty {
                source.series_mut().calculate_stats();
            }
        }
        for source in &self.sources {
            if source.series().stats.max > max_activity_value {
                max_activity_value = source.series().stats.max;
            }
            if source.series().stats.min < min_activity_value {
                min_activity_value = source.series().stats.min;
            }
            sum_activity_values += source.series().stats.sum;
            filled_stats += 1;
        }
        // Account for the decoration requested height
        for decoration in &self.decorations {
            let top_value = decoration.top_value();
            let bottom_value = decoration.bottom_value();
            if top_value > max_activity_value {
                max_activity_value = top_value
            }
            if bottom_value < min_activity_value {
                min_activity_value = bottom_value;
            }
        }
        self.stats.max = max_activity_value;
        self.stats.min = min_activity_value;
        self.stats.sum = sum_activity_values;
        self.stats.avg = sum_activity_values / filled_stats as f64;
        self.stats.is_dirty = false;
        debug!("Chart: Updated statistics to: {:?}", self.stats);
    }
}

impl Default for TimeSeries {
    /// `new` returns the default
    fn default() -> TimeSeries {
        // This leads to 5 mins of metrics to show by default.
        let default_capacity = 300usize;
        TimeSeries {
            metrics_capacity: default_capacity,
            metrics: Vec::with_capacity(default_capacity),
            stats: TimeSeriesStats::default(),
            collision_policy: ValueCollisionPolicy::default(),
            missing_values_policy: MissingValuesPolicy::default(),
            first_idx: 0,
            last_idx: 0,
            active_items: 0,
        }
    }
}

impl TimeSeries {
    /// `with_capacity` builder changes the amount of metrics in the vec
    pub fn with_capacity(self, n: usize) -> TimeSeries {
        let mut new_self = self;
        new_self.metrics = Vec::with_capacity(n);
        new_self.metrics_capacity = n;
        new_self
    }

    /// `with_missing_values_policy` receives a String and returns
    /// a MissingValuesPolicy, TODO: the "Fixed" value is not implemented.
    pub fn with_missing_values_policy(mut self, policy_type: String) -> TimeSeries {
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

    /// `calculate_stats` Iterates over the metrics and sets the stats
    pub fn calculate_stats(&mut self) {
        // Recalculating seems to be necessary because we are constantly
        // moving items out of the Vec<> so our cache can easily get out of
        // sync
        let mut max_activity_value = std::f64::MIN;
        let mut min_activity_value = std::f64::MAX;
        let mut sum_activity_values = 0f64;
        let mut filled_metrics = 0usize;
        for entry in self.iter() {
            if let Some(metric) = entry.1 {
                if metric > max_activity_value {
                    max_activity_value = metric;
                }
                if metric < min_activity_value {
                    min_activity_value = metric;
                }
                sum_activity_values += metric;
                filled_metrics += 1;
            }
        }
        self.stats.max = max_activity_value;
        self.stats.min = min_activity_value;
        self.stats.sum = sum_activity_values;
        self.stats.avg = sum_activity_values / (filled_metrics as f64);
        self.stats.is_dirty = false;
    }

    /// `get_missing_values_fill` uses the MissingValuesPolicy to decide
    /// which value to place on empty metric timeslots when drawing
    pub fn get_missing_values_fill(&self) -> f64 {
        match self.missing_values_policy {
            MissingValuesPolicy::Zero => 0f64,
            MissingValuesPolicy::One => 1f64,
            MissingValuesPolicy::Min => self.stats.min,
            MissingValuesPolicy::Max => self.stats.max,
            MissingValuesPolicy::Last => self.get_last_filled(),
            MissingValuesPolicy::First => self.get_first_filled(),
            MissingValuesPolicy::Avg => self.stats.avg,
            MissingValuesPolicy::Fixed(val) => val,
        }
    }

    /// `resolve_metric_collision` ensures the policy for colliding values is
    /// applied.
    pub fn resolve_metric_collision(&self, existing: f64, new: f64) -> f64 {
        match self.collision_policy {
            ValueCollisionPolicy::Increment => existing + new,
            ValueCollisionPolicy::Overwrite => new,
            ValueCollisionPolicy::Decrement => existing - new,
            ValueCollisionPolicy::Ignore => existing,
        }
    }

    /// `circular_push` an item to the circular buffer
    pub fn circular_push(&mut self, input: (u64, Option<f64>)) {
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
        self.stats.is_dirty = true;
        self.last_idx = (self.last_idx + 1) % (self.metrics_capacity + 1);
    }

    /// `push` Adds values to the circular buffer adding empty entries for
    /// missing entries, may invalidate the buffer if all data is outdated
    pub fn push(&mut self, input: (u64, f64)) {
        if !self.metrics.is_empty() {
            let last_idx = if self.last_idx == self.metrics_capacity || self.last_idx == 0 {
                self.metrics.len() - 1
            } else {
                self.last_idx - 1
            };
            let inactive_time = if input.0 > self.metrics[last_idx].0 {
                (input.0 - self.metrics[last_idx].0) as usize
            } else {
                0usize
            };
            if inactive_time > self.metrics_capacity {
                // The whole vector should be discarded
                self.first_idx = 0;
                self.last_idx = 1;
                self.metrics[0] = (input.0, Some(input.1));
                self.active_items = 1;
            } else if inactive_time == 0 {
                // In this case, the last epoch and the current epoch match
                if let Some(curr_val) = self.metrics[last_idx].1 {
                    self.metrics[last_idx].1 =
                        Some(self.resolve_metric_collision(curr_val, input.1));
                } else {
                    self.metrics[last_idx].1 = Some(input.1);
                }
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
    pub fn get_last_filled(&self) -> f64 {
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
        0f64
    }

    /// `get_first_filled` Returns the first filled entry in the circular buffer
    pub fn get_first_filled(&self) -> f64 {
        for entry in self.iter() {
            if let Some(metric) = entry.1 {
                return metric;
            }
        }
        0f64
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
    pub fn as_vec(&self) -> Vec<(u64, Option<f64>)> {
        if self.metrics.is_empty() {
            return vec![];
        }
        let mut res: Vec<(u64, Option<f64>)> = Vec::with_capacity(self.metrics_capacity);
        for entry in self.iter() {
            res.push(entry.clone());
        }
        res
    }

    fn push_current_epoch(&mut self, input: f64) {
        let now = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.push((now, input));
    }

    // `iter` Returns an Iterator from the current start.
    fn iter(&self) -> IterTimeSeries {
        IterTimeSeries {
            inner: self,
            pos: self.first_idx,
            current_item: 0,
        }
    }
}

impl<'a> Iterator for IterTimeSeries<'a> {
    type Item = &'a (u64, Option<f64>);
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
        test.circular_push((10, Some(0f64)));
        assert_eq!(test.first_idx, 0);
        assert_eq!(test.last_idx, 1);
        test.circular_push((11, Some(1f64)));
        test.circular_push((12, None));
        test.circular_push((13, Some(3f64)));
        assert_eq!(test.first_idx, 0);
        assert_eq!(test.last_idx, 4);
        assert_eq!(
            test.metrics,
            vec![
                (10, Some(0f64)),
                (11, Some(1f64)),
                (12, None),
                (13, Some(3f64))
            ]
        );
        test.circular_push((14, Some(4f64)));
        assert_eq!(
            test.metrics,
            vec![
                (14, Some(4f64)),
                (11, Some(1f64)),
                (12, None),
                (13, Some(3f64))
            ]
        );
        assert_eq!(test.first_idx, 1);
        assert_eq!(test.last_idx, 0);
        test.circular_push((15, Some(5f64)));
        assert_eq!(
            test.metrics,
            vec![
                (14, Some(4f64)),
                (15, Some(5f64)),
                (12, None),
                (13, Some(3f64))
            ]
        );
        assert_eq!(test.first_idx, 2);
        assert_eq!(test.last_idx, 1);
    }
    #[test]
    fn it_gets_last_filled_value() {
        let mut test = TimeSeries::default().with_capacity(4);
        // Some values should be inserted as None
        test.push((10, 0f64));
        test.circular_push((11, None));
        test.circular_push((12, None));
        test.circular_push((13, None));
        assert_eq!(test.get_last_filled(), 0f64);
        let mut test = TimeSeries::default().with_capacity(4);
        test.circular_push((11, None));
        test.push((12, 2f64));
    }
    #[test]
    fn it_transforms_to_flat_vec() {
        let mut test = TimeSeries::default().with_capacity(4);
        // Some values should be inserted as None
        assert_eq!(test.first_idx, 0);
        assert_eq!(test.last_idx, 0);
        test.push((10, 0f64));
        assert_eq!(test.first_idx, 0);
        assert_eq!(test.last_idx, 1);
        test.push((13, 3f64));
        assert_eq!(test.first_idx, 0);
        assert_eq!(test.last_idx, 4);
        assert_eq!(
            test.as_vec(),
            vec![(10, Some(0f64)), (11, None), (12, None), (13, Some(3f64))]
        );
        test.push((14, 4f64));
        // Starting at 11
        test.first_idx = 1;
        test.last_idx = 1;
        assert_eq!(
            test.as_vec(),
            vec![(11, None), (12, None), (13, Some(3f64)), (14, Some(4f64))]
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
        assert_eq!(test.as_vec(), vec![(13, Some(3f64))]);
        // 13, 14
        test.first_idx = 3;
        test.last_idx = 1;
        test.active_items = 2;
        assert_eq!(test.as_vec(), vec![(13, Some(3f64)), (14, Some(4f64))]);
    }
    #[test]
    fn it_fills_empty_epochs() {
        let mut test = TimeSeries::default().with_capacity(4);
        // Some values should be inserted as None
        test.push((10, 0f64));
        test.push((13, 3f64));
        assert_eq!(
            test.metrics,
            vec![(10, Some(0f64)), (11, None), (12, None), (13, Some(3f64))]
        );
        assert_eq!(test.active_items, 4);
        // Test the whole vector is discarded
        test.push((18, 8f64));
        assert_eq!(test.active_items, 1);
        assert_eq!(
            test.metrics,
            vec![(18, Some(8f64)), (11, None), (12, None), (13, Some(3f64))]
        );
        assert_eq!(test.first_idx, 0);
        assert_eq!(test.last_idx, 1);
        assert_eq!(test.active_items, 1);
        assert_eq!(test.as_vec(), vec![(18, Some(8f64))]);
        test.push((20, 0f64));
        assert_eq!(
            test.metrics,
            vec![
                (18, Some(8f64)),
                (19, None),
                (20, Some(0f64)),
                (13, Some(3f64))
            ]
        );
        assert_eq!(test.first_idx, 0);
        assert_eq!(test.last_idx, 3);
        assert_eq!(test.active_items, 3);
        assert_eq!(
            test.as_vec(),
            vec![(18, Some(8f64)), (19, None), (20, Some(0f64))]
        );
        test.push((50, 5f64));
        assert_eq!(
            test.metrics,
            // Many outdated entries
            vec![
                (50, Some(5f64)),
                (19, None),
                (20, Some(0f64)),
                (13, Some(3f64))
            ]
        );
        assert_eq!(test.as_vec(), vec![(50, Some(5f64))]);
        test.push((53, 3f64));
        assert_eq!(
            test.metrics,
            vec![(50, Some(5f64)), (51, None), (52, None), (53, Some(3f64))]
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
        test_zero.push((0, 9f64));
        test_zero.push((2, 1f64));
        test_one.push((0, 9f64));
        test_one.push((2, 1f64));
        test_min.push((0, 9f64));
        test_min.push((2, 1f64));
        test_max.push((0, 9f64));
        test_max.push((2, 1f64));
        test_last.push((0, 9f64));
        test_last.push((2, 1f64));
        test_first.push((0, 9f64));
        test_first.push((2, 1f64));
        test_avg.push((0, 9f64));
        test_avg.push((2, 1f64));
        test_zero.calculate_stats();
        test_one.calculate_stats();
        test_min.calculate_stats();
        test_max.calculate_stats();
        test_last.calculate_stats();
        test_first.calculate_stats();
        test_avg.calculate_stats();
        assert_eq!(test_zero.get_missing_values_fill(), 0f64);
        assert_eq!(test_one.get_missing_values_fill(), 1f64);
        assert_eq!(test_min.get_missing_values_fill(), 1f64);
        assert_eq!(test_max.get_missing_values_fill(), 9f64);
        assert_eq!(test_last.get_missing_values_fill(), 1f64);
        assert_eq!(test_first.get_missing_values_fill(), 9f64);
        assert_eq!(test_avg.get_missing_values_fill(), 5f64);
        // TODO: add Fixed value test
    }
    #[test]
    fn it_iterates_trait() {
        // Iterator Trait
        // Test an empty TimeSeries vec
        let test0: TimeSeries = TimeSeries::default().with_capacity(4);
        let mut iter_test0 = test0.iter();
        assert_eq!(iter_test0.pos, 0);
        assert!(iter_test0.next().is_none());
        assert!(iter_test0.next().is_none());
        assert_eq!(iter_test0.pos, 0);
        // Simple test with one item
        let mut test1 = TimeSeries::default().with_capacity(4);
        test1.circular_push((10, Some(0f64)));
        let mut iter_test1 = test1.iter();
        assert_eq!(iter_test1.next(), Some(&(10, Some(0f64))));
        assert_eq!(iter_test1.pos, 1);
        assert!(iter_test1.next().is_none());
        assert!(iter_test1.next().is_none());
        assert_eq!(iter_test1.pos, 1);
        // Simple test with 3 items, rotated to start first item and 2nd
        // position and last item at 3rd position
        let mut test2 = TimeSeries::default().with_capacity(4);
        test2.circular_push((10, Some(0f64)));
        test2.circular_push((11, Some(1f64)));
        test2.circular_push((12, Some(2f64)));
        test2.circular_push((13, Some(3f64)));
        test2.first_idx = 1;
        test2.last_idx = 3;
        assert_eq!(
            test2.metrics,
            vec![
                (10, Some(0f64)),
                (11, Some(1f64)),
                (12, Some(2f64)),
                (13, Some(3f64))
            ]
        );
        let mut iter_test2 = test2.iter();
        assert_eq!(iter_test2.pos, 1);
        assert_eq!(iter_test2.next(), Some(&(11, Some(1f64))));
        assert_eq!(iter_test2.next(), Some(&(12, Some(2f64))));
        assert_eq!(iter_test2.pos, 3);
        // A vec that is completely full
        let mut test3 = TimeSeries::default().with_capacity(4);
        test3.circular_push((10, Some(0f64)));
        test3.circular_push((11, Some(1f64)));
        test3.circular_push((12, Some(2f64)));
        test3.circular_push((13, Some(3f64)));
        {
            let mut iter_test3 = test3.iter();
            assert_eq!(iter_test3.next(), Some(&(10, Some(0f64))));
            assert_eq!(iter_test3.next(), Some(&(11, Some(1f64))));
            assert_eq!(iter_test3.next(), Some(&(12, Some(2f64))));
            assert_eq!(iter_test3.next(), Some(&(13, Some(3f64))));
            assert!(iter_test3.next().is_none());
            assert!(iter_test3.next().is_none());
            assert_eq!(iter_test2.pos, 3);
        }
        // After changing the data the idx is recreatehd at 11 as expected
        test3.circular_push((14, Some(4f64)));
        let mut iter_test3 = test3.iter();
        assert_eq!(iter_test3.next(), Some(&(11, Some(1f64))));
    }

    #[test]
    fn it_scales_x_to_display_size() {
        let mut test = SizeInfo {
            padding_x: 0.,
            padding_y: 0.,
            height: 100.,
            width: 100.,
            ..SizeInfo::default()
        };
        // display size: 100 px, input the value: 0, padding_x: 0
        // The value should return should be left-most: -1.0
        let min = test.scale_x(0f32);
        assert_eq!(min, -1.0f32);
        // display size: 100 px, input the value: 100, padding_x: 0
        // The value should return should be right-most: 1.0
        let max = test.scale_x(100f32);
        assert_eq!(max, 1.0f32);
        // display size: 100 px, input the value: 50, padding_x: 0
        // The value should return should be the center: 0.0
        let mid = test.scale_x(50f32);
        assert_eq!(mid, 0.0f32);
        test.padding_x = 50.;
        // display size: 100 px, input the value: 50, padding_x: 50px
        // The value returned should be the right-most: 1.0
        let mid = test.scale_x(50f32);
        assert_eq!(mid, 1.0f32);
    }

    #[test]
    fn it_scales_y_to_display_size() {
        let mut size_test = SizeInfo {
            padding_x: 0.,
            padding_y: 0.,
            height: 100.,
            ..SizeInfo::default()
        };
        let mut chart_test = TimeSeriesChart::default();
        // To make testing easy, let's make three values equivalent:
        // - Chart height
        // - Max Metric collected
        // - Max resolution in pixels
        chart_test.stats.max = 100f64;
        // display size: 100 px, input the value: 0, padding_y: 0
        // The value should return should be lowest: -1.0
        let min = size_test.scale_y(100f64, 0f64);
        assert_eq!(min, -1.0f32);
        // display size: 100 px, input the value: 100, padding_y: 0
        // The value should return should be upper-most: 1.0
        let max = size_test.scale_y(100f64, 100f64);
        assert_eq!(max, 1.0f32);
        // display size: 100 px, input the value: 50, padding_y: 0
        // The value should return should be the center: 0.0
        let mid = size_test.scale_y(100f64, 50f64);
        assert_eq!(mid, 0.0f32);
        size_test.padding_y = 25.;
        // display size: 100 px, input the value: 50, padding_y: 25
        // The value returned should be upper-most: 1.0
        // In this case, the chart (100px) is bigger than the display,
        // which means some values would have been chopped (anything above
        // 50f32)
        let mid = size_test.scale_y(100f64, 50f64);
        assert_eq!(mid, 1.0f32);
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
// `with_marker_line` initializes the marker line into a Some
// pub fn with_marker_line(mut self, value: T) -> ActivityLevels<T> {
// self.marker_line = Some(value);
// self
// }
//
//
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
