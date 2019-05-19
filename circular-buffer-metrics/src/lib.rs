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
// -- mock the prometheus server and response

extern crate futures;
extern crate hyper;
extern crate num_traits;
extern crate serde;
extern crate serde_json;
extern crate tokio_core;
// use crate::term::color::Rgb;
// use crate::term::SizeInfo;
use num_traits::*;
use std::collections::HashMap;
use std::time::UNIX_EPOCH;

#[macro_use]
extern crate serde_derive;
use hyper::rt::{Future, Stream};
use hyper::Client;

/// `MissingValuesPolicy` provides several ways to deal with missing values
/// when drawing the Metric
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

/// `IterTimeSeries` provides the Iterator Trait for TimeSeries metrics.
/// The state for the iteration is held en "pos" field. The "current_item" is
/// used to determine if further iterations on the circular buffer is needed.
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

// The below data structures for parsing something like:
//  {
//   "data": {
//     "result": [
//       {
//         "metric": {
//           "__name__": "up",
//           "instance": "localhost:9090",
//           "job": "prometheus"
//         },
//         "value": [
//           1557052757.816,
//           "1"
//         ]
//       },{...}
//     ],
//     "resultType": "vector"
//   },
//   "status": "success"
// }

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum PrometheusResult {
    Vector(PrometheusVectorResult),
    Matrix(PrometheusMatrixResult),
}

/// Implements the Default PrometheusResult
impl Default for PrometheusResult {
    fn default() -> PrometheusResult {
        PrometheusResult::Vector(PrometheusVectorResult {
            labels: HashMap::new(),
            value: vec![],
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
struct PrometheusMatrixResult {
    #[serde(rename = "metric")]
    labels: HashMap<String, String>,
    values: Vec<Vec<serde_json::Value>>,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
struct PrometheusVectorResult {
    #[serde(rename = "metric")]
    labels: HashMap<String, String>,
    value: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum PrometheusResultType {
    #[serde(rename = "matrix")]
    Matrix,
    #[serde(rename = "vector")]
    Vector,
    #[serde(rename = "scalar")]
    Scalar,
    #[serde(rename = "string")]
    String,
    Unknown,
}

/// Implements the Default ResultType from Prometheus.
/// https://prometheus.io/docs/prometheus/latest/querying/api/#expression-query-result-formats
impl Default for PrometheusResultType {
    fn default() -> PrometheusResultType {
        PrometheusResultType::Unknown
    }
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
struct PrometheusResponseData {
    result: Vec<PrometheusResult>,
    #[serde(rename = "resultType")]
    result_type: PrometheusResultType,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct PrometheusResponse {
    data: PrometheusResponseData,
    status: String,
}

#[derive(Clone, Debug)]
pub struct PrometheusTimeSeries<'a, T>
where
    T: Num + Clone + Copy,
{
    /// The TimeSeries metrics storage
    pub time_series: TimeSeries<T>,

    /// The URL were Prometheus metrics may be acquaired
    pub url: hyper::Uri,

    /// A response may be vector, matrix, scalar or string
    pub result_type: PrometheusResultType,

    /// The Labels key and value, if any, to match the response
    pub labels: HashMap<String, String>,

    /// The time in secondso to get the metrics from Prometheus
    /// Shouldn't be faster than the scrape interval for the Target
    pub pull_interval: usize,

    /// Tokio Core Handle
    pub tokio_core: &'a tokio_core::reactor::Handle,
}

impl<'a, T> PrometheusTimeSeries<'a, T>
where
    T: Num + Clone + Copy + std::marker::Send,
{
    /// `new` returns a new PrometheusTimeSeries. it takes a URL where to load
    /// the data from and a pull_interval, this should match scrape interval in
    /// Prometheus Server side to avoid pulling the same values over and over.
    /// A tokio_core handle must be passed to the constructor to be used for
    /// asynchronous tasks
    pub fn new(
        url: String,
        pull_interval: usize,
        result_type: String,
        labels: HashMap<String, String>,
        tokio_core: &tokio_core::reactor::Handle,
    ) -> Result<PrometheusTimeSeries<T>, String> {
        //url should be like ("http://localhost:9090/api/v1/query?{}",query)
        let prom_result_type = match result_type.as_ref() {
            "vector" => PrometheusResultType::Vector,
            "matrix" => PrometheusResultType::Matrix,
            "scalar" => PrometheusResultType::Scalar,
            "string" => PrometheusResultType::String,
            _ => PrometheusResultType::Unknown,
        };
        if prom_result_type == PrometheusResultType::Unknown {
            return Err(format!("Unknown ResultType '{}'", result_type));
        }
        match url.parse::<hyper::Uri>() {
            Ok(url) => {
                if url.scheme_part() == Some(&hyper::http::uri::Scheme::HTTP) {
                    Ok(PrometheusTimeSeries {
                        time_series: TimeSeries::default(),
                        url,
                        pull_interval,
                        result_type: prom_result_type,
                        tokio_core,
                        labels,
                    })
                } else {
                    Err(String::from("Only http is supported."))
                }
            }
            Err(_) => Err(String::from("Invalid URL")),
        }
    }

    /// `match_metric_labels` checks the labels in the incoming
    /// PrometheusResponseData contains the required labels
    pub fn match_metric_labels(&self, metric_labels: &HashMap<String, String>) -> bool {
        for (required_label, required_value) in &self.labels {
            match metric_labels.get(required_label) {
                Some(return_value) => {
                    if return_value != required_value {
                        println!("Required label {} exists but required value: {} does not match existing value: {}", required_label, required_value, return_value);
                        return false;
                    } else {
                        println!(
                            "Required label {} exists and matches required value",
                            required_label
                        );
                    }
                }
                None => {
                    println!("Required label {} does not exists", required_label);
                    return false;
                }
            }
        }
        true
    }

    /// Transforms an serde_json::Value into an optional T
    pub fn serde_json_to_num(&self, input: &serde_json::Value) -> Option<T>
    where
        T: Num + FromPrimitive + Bounded + ToPrimitive + PartialOrd,
    {
        if input.is_string() {
            if let Some(input) = input.as_str() {
                if let Ok(value) = T::from_str_radix(input, 10) {
                    return Some(value);
                }
            }
        }
        None
    }

    /// Transforms an serde_json::Value into an optional u64
    /// The epoch coming from Prometheus is a float (epoch with millisecond),
    /// but our internal representation is u64
    pub fn prometheus_epoch_to_u64(&self, input: &serde_json::Value) -> Option<u64> {
        if input.is_number() {
            if let Some(input) = input.as_f64() {
                return Some(input as u64);
            }
        }
        None
    }

    /// `load_prometheus_response` loads data from PrometheusResponse into
    /// the internal `time_series`, returns the number of items or an error
    /// string
    pub fn load_prometheus_response(&mut self, res: PrometheusResponse) -> Result<usize, String>
    where
        T: FromPrimitive + Bounded + ToPrimitive + PartialOrd + std::fmt::Debug,
    {
        let mut loaded_items = 0;
        if res.status != "success" {
            return Ok(0usize);
        }
        match res.data.result_type {
            PrometheusResultType::Vector => {
                if let PrometheusResult::Vector(result) = res.data.result {
                    for item in res.data.result.iter() {
                        if self.match_metric_labels(&item.labels) {
                            // The result array is  [epoch, value, epoch, value]
                            for item in item.value.chunks(2) {
                                let opt_epoch = self.prometheus_epoch_to_u64(&item[0]);
                                let opt_value = self.serde_json_to_num(&item[1]);
                                if let (Some(epoch), Some(value)) = (opt_epoch, opt_value) {
                                    self.time_series.push((epoch, value));
                                    loaded_items += 1;
                                }
                            }
                        }
                    }
                }
            }
            _ => println!("Skipping result_type: {:?}", res.data.result_type),
        };
        if loaded_items > 0 {
            self.time_series.calculate_stats();
        }
        Ok(loaded_items)
    }

    /// `get_from_prometheus` is an async operation that returns an Optional
    /// PrometheusResponse, this function uses the internal `tokio_core`'s
    /// handle which is a runtime provided to the class, the body of the
    /// response is parsed and returned eventually.
    pub fn get_from_prometheus<'b>(
        &mut self,
    ) -> impl Future<Item = Option<PrometheusResponse>, Error = ()> + 'b {
        Client::new()
            .get(self.url.clone())
            .and_then(|res| {
                println!("Response: {}", res.status());
                println!("Headers: {:?}", res.headers());
                res.into_body()
                    // A hyper::Body is a Stream of Chunk values. We need a
                    // non-blocking way to get all the chunks so we can deserialize the response.
                    // The concat2() function takes the separate body chunks and makes one
                    // hyper::Chunk value with the contents of the entire body
                    .concat2()
                    .and_then(|body| Ok(parse_json(&body)))
            })
            .map_err(|err| {
                println!("Error: {}", err);
            })
    }
}

/// `parse_json` transforms a hyper body chunk into a possible
/// PrometheusResponse, mostly used for testing
pub fn parse_json(body: &hyper::Chunk) -> Option<PrometheusResponse> {
    let prom_res: Result<PrometheusResponse, serde_json::Error> = serde_json::from_slice(&body);
    // XXX: Figure out how to return the error
    match prom_res {
        Ok(v) => {
            println!("returned JSON: {:?}", v);
            Some(v)
        }
        Err(err) => {
            println!("Unable to parse JSON: {:?}", err);
            None
        }
    }
}
/// Implement PartialEq for PrometheusTimeSeries because we should ignore
/// tokio_core_handle
impl<'a, L> PartialEq<PrometheusTimeSeries<'a, L>> for PrometheusTimeSeries<'a, L>
where
    L: Num + Copy,
{
    fn eq(&self, other: &PrometheusTimeSeries<L>) -> bool
    where
        L: Num + Copy,
    {
        self.time_series == other.time_series
            && self.url == other.url
            && self.pull_interval == other.pull_interval
    }
}

/// `TimeSeriesChart` has an array of TimeSeries to display, it contains the
/// X, Y position and has methods to draw in opengl.
#[derive(Default, Debug)]
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
    pub chart_height: f32,

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

impl<T> TimeSeriesChart<T>
where
    T: Num + Clone + Copy,
{
    /// `scale_x_to_size` Scales the value from the current display boundary to
    /// a cartesian plane from [-1.0, 1.0], where -1.0 is 0px (left-most) and
    /// 1.0 is the `display_width` parameter (right-most), i.e. 1024px.
    pub fn scale_x_to_size(&self, input_value: T, display_width: f32, padding_x: f32) -> f32
    where
        T: Num + ToPrimitive,
    {
        let center_x = display_width / 2.;
        let x = padding_x + self.x_offset + input_value.to_f32().unwrap();
        (x - center_x) / center_x
    }
    /// `scale_y_to_size` Scales the value from the current display boundary to
    /// a cartesian plane from [-1.0, 1.0], where 1.0 is 0px (top) and -1.0 is
    /// the `display_height` parameter (bottom), i.e. 768px.
    pub fn scale_y_to_size(&self, input_value: T, display_height: f32, padding_y: f32) -> f32
    where
        T: Num + ToPrimitive,
    {
        let center_y = display_height / 2.;
        let y = display_height
            - 2. * padding_y
            - (self.chart_height * num_traits::ToPrimitive::to_f32(&input_value).unwrap()
                / num_traits::ToPrimitive::to_f32(&self.metrics.metric_stats.max).unwrap());
        -(y - center_y) / center_y
    }
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
    fn iter(&self) -> IterTimeSeries<T>
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
    use tokio_core::reactor::Core;

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

    #[test]
    fn it_skips_prometheus_errors() {
        // Create a Tokio Core to use for testing
        let core = Core::new().unwrap();
        let core_handle = &core.handle();
        // This URL has the end time BEFORE the start time
        let test0_res: Result<PrometheusTimeSeries<f32>, String> = PrometheusTimeSeries::new(
            String::from("http://localhost:9090/api/v1/query_range?query=node_load1&start=1558253499&end=1558253479&step=1"),
            15,
            String::from("matrix"),
            HashMap::new(),
            &core_handle,
        );
        assert_eq!(test0_res.is_ok(), true);
        let mut test0 = test0_res.unwrap();
        // A json returned by prometheus
        let test0_json = hyper::Chunk::from(
            r#"
            {
              "status": "error",
              "errorType": "bad_data",
              "error": "end timestamp must not be before start time"
            }
            "#,
        );
        let res0_json = parse_json(&test0_json);
        assert_eq!(res0_json.is_none(), true);
    }
    #[test]
    fn it_loads_prometheus_matrix() {
        // Create a Tokio Core to use for testing
        let core = Core::new().unwrap();
        let core_handle = &core.handle();
        let test0_res: Result<PrometheusTimeSeries<f32>, String> = PrometheusTimeSeries::new(
            String::from("http://localhost:9090/api/v1/query_range?query=node_load1&start=1558253469&end=1558253479&step=1"),
            15,
            String::from("matrix"),
            HashMap::new(),
            &core_handle,
        );
        assert_eq!(test0_res.is_ok(), true);
        let mut test0 = test0_res.unwrap();
        // A json returned by prometheus
        let test0_json = hyper::Chunk::from(
            r#"
            {
              "status": "success",
              "data": {
                "resultType": "matrix",
                "result": [
                  {
                    "metric": {
                      "__name__": "node_load1",
                      "instance": "localhost:9100",
                      "job": "node_exporter"
                    },
                    "values": [
                        [1558253469,"1.42"],[1558253470,"1.42"],[1558253471,"1.55"],
                        [1558253472,"1.55"],[1558253473,"1.55"],[1558253474,"1.55"],
                        [1558253475,"1.55"],[1558253476,"1.55"],[1558253477,"1.55"],
                        [1558253478,"1.55"],[1558253479,"1.55"]]
                  }
                ]
              }
            }"#,
        );
        let res0_json = parse_json(&test0_json);
        assert_eq!(res0_json.is_some(), true);
        let res0_load = test0.load_prometheus_response(res0_json.clone().unwrap());
        // 2 items should have been loaded, one for Prometheus Server and the
        // other for Prometheus Node Exporter
        assert_eq!(res0_load, Ok(11usize));
    }
    #[test]
    fn it_loads_prometheus_vector() {
        // Create a Tokio Core to use for testing
        let core = Core::new().unwrap();
        let core_handle = &core.handle();
        let mut metric_labels = HashMap::new();
        let test0_res: Result<PrometheusTimeSeries<f32>, String> = PrometheusTimeSeries::new(
            String::from("http://localhost:9090/api/v1/query?query=up"),
            15,
            String::from("vector"),
            metric_labels.clone(),
            &core_handle,
        );
        assert_eq!(test0_res.is_ok(), true);
        let mut test0 = test0_res.unwrap();
        // A json returned by prometheus
        let test0_json = hyper::Chunk::from(
            r#"
            {
              "status": "success",
              "data": {
                "resultType": "vector",
                "result": [
                  {
                    "metric": {
                      "__name__": "up",
                      "instance": "localhost:9090",
                      "job": "prometheus"
                    },
                    "value": [
                      1557571137.732,
                      "1"
                    ]
                  },
                  {
                    "metric": {
                      "__name__": "up",
                      "instance": "localhost:9100",
                      "job": "node_exporter"
                    },
                    "value": [
                      1557571137.732,
                      "1"
                    ]
                  }
                ]
              }
            }"#,
        );
        let res0_json = parse_json(&test0_json);
        assert_eq!(res0_json.is_some(), true);
        let res0_load = test0.load_prometheus_response(res0_json.clone().unwrap());
        // 2 items should have been loaded, one for Prometheus Server and the
        // other for Prometheus Node Exporter
        assert_eq!(res0_load, Ok(2usize));

        // Make the labels match only one instance
        metric_labels.insert(String::from("job"), String::from("prometheus"));
        metric_labels.insert(String::from("instance"), String::from("localhost:9090"));
        test0.labels = metric_labels.clone();
        let res1_load = test0.load_prometheus_response(res0_json.clone().unwrap());
        assert_eq!(res1_load, Ok(1usize));

        // Make the labels not match
        metric_labels.insert(String::from("__name__"), String::from("down"));
        test0.labels = metric_labels.clone();
        let res2_load = test0.load_prometheus_response(res0_json.clone().unwrap());
        assert_eq!(res2_load, Ok(0usize));
        // By default the metrics should have been Incremented (ValueCollisionPolicy)
        // We have imported the metric 3 times
        assert_eq!(test0.time_series.as_vec(), vec![(1557571137u64, Some(3.))]);
    }

    #[test]
    fn it_gets_prometheus_metrics() {
        // Create a Tokio Core to use for testing
        let mut core = Core::new().unwrap();
        let mut test_labels = HashMap::new();
        test_labels.insert(String::from("name"), String::from("up"));
        test_labels.insert(String::from("job"), String::from("prometheus"));
        test_labels.insert(String::from("instance"), String::from("localhost:9090"));
        let core_handle = &core.handle();
        // Test non plain http error:
        let test0_res: Result<PrometheusTimeSeries<f32>, String> = PrometheusTimeSeries::new(
            String::from("https://localhost:9090/api/v1/query?query=up"),
            15,
            String::from("vector"),
            test_labels.clone(),
            &core_handle,
        );
        assert_eq!(test0_res, Err(String::from("Only http is supported.")));
        let test1_res: Result<PrometheusTimeSeries<f32>, String> = PrometheusTimeSeries::new(
            String::from("http://localhost:9090/api/v1/query?query=up"),
            15,
            String::from("vector"),
            test_labels.clone(),
            &core_handle,
        );
        assert_eq!(test1_res.is_ok(), true);
        let mut test1 = test1_res.unwrap();
        let res1_get = core.run(test1.get_from_prometheus());
        assert_eq!(res1_get.is_ok(), true);
        if let Ok(Some(prom_response)) = res1_get {
            // This requires a Prometheus Server running locally
            // XXX: mock this.
            assert_eq!(prom_response.status, String::from("success"));
            assert_eq!(prom_response.data.result_type, PrometheusResultType::Vector);
            let prom_data_result = prom_response.data.result;
            assert_ne!(prom_data_result.len(), 0);
            // PrometheusResult { metric: PrometheusMetricName { name: "up", instance: "localhost:9090", job: "prometheus" }, value: [Number(1557246907.503), String("1")] }
            let mut found_prometheus_job_metric = false;
            for prom_item in prom_data_result.iter() {
                if test1.match_metric_labels(&test_labels) {
                    assert_eq!(prom_item.value.len(), 2);
                    assert_eq!(prom_item.value[1], String::from("1"));
                    found_prometheus_job_metric = true;
                }
            }
            assert_eq!(found_prometheus_job_metric, true);
        }
    }

    #[test]
    fn it_scales_x_to_display_size() {
        let test = TimeSeriesChart::default();
        // display size: 100 px, input the value: 0, padding_x: 0
        // The value should return should be left-most: -1.0
        let min = test.scale_x_to_size(0f32, 100f32, 0f32);
        assert_eq!(min, -1.0f32);
        // display size: 100 px, input the value: 100, padding_x: 0
        // The value should return should be right-most: 1.0
        let max = test.scale_x_to_size(100f32, 100f32, 0f32);
        assert_eq!(max, 1.0f32);
        // display size: 100 px, input the value: 50, padding_x: 0
        // The value should return should be the center: 0.0
        let mid = test.scale_x_to_size(50f32, 100f32, 0f32);
        assert_eq!(mid, 0.0f32);
        // display size: 100 px, input the value: 50, padding_x: 50px
        // The value returned should be the right-most: 1.0
        let mid = test.scale_x_to_size(50f32, 100f32, 50f32);
        assert_eq!(mid, 1.0f32);
    }

    #[test]
    fn it_scales_y_to_display_size() {
        let mut test = TimeSeriesChart::default();
        // To make testing easy, let's make three values equivalent:
        // - Chart height
        // - Max Metric collected
        // - Max resolution in pixels
        test.metrics.metric_stats.max = 100f32;
        test.chart_height = 100f32;
        // display size: 100 px, input the value: 100, padding_y: 0
        // The value should return should be lowest: -1.0
        println!("Checking TimeSeries: {:?}", test);
        let min = test.scale_y_to_size(0f32, 100f32, 0f32);
        assert_eq!(min, -1.0f32);
        // display size: 100 px, input the value: 100, padding_y: 0
        // The value should return should be upper-most: 1.0
        let max = test.scale_y_to_size(100f32, 100f32, 0f32);
        assert_eq!(max, 1.0f32);
        // display size: 100 px, input the value: 50, padding_y: 0
        // The value should return should be the center: 0.0
        let mid = test.scale_y_to_size(50f32, 100f32, 0f32);
        assert_eq!(mid, 0.0f32);
        // display size: 100 px, input the value: 50, padding_y: 25
        // The value returned should be upper-most: 1.0
        // In this case, the chart (100px) is bigger than the display,
        // which means some values would have been chopped (anything above
        // 50f32)
        let mid = test.scale_y_to_size(50f32, 100f32, 25f32);
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
