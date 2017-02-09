//! The cache module dictates how metrics will be buffered before being sent to the
//! corresponding backend.

use std::collections::{hash_map, HashMap};
use std::rc::Rc;

use parse::{Metric, MetricType};

/// `CapellaCache` is the bucketing mechanism used by capella to buffer metrics before sending to
/// the backend.
#[derive(Debug, Default)]
pub struct CapellaCache {
    counters: HashMap<Rc<String>, f64>,
    gauges: HashMap<Rc<String>, f64>,
    timers: HashMap<Rc<String>, Vec<f64>>,
    timer_data: HashMap<String, f64>,
    metrics_seen: u64,
    bad_metrics: u64,
}

impl CapellaCache {
    /// This function will add a `Metric` to the cache.
    pub fn add_metric(&mut self, metric: &Metric) {
        self.metrics_seen += 1;

        match metric.metric_type {
            MetricType::Counter => {
                let c = self.counters.entry(metric.name.clone()).or_insert(0.0);
                if let Some(rate) = metric.sample_rate {
                    *c += metric.value * rate;
                }
                *c += metric.value;
            },
            MetricType::Gauge => {
                self.gauges.insert(metric.name.clone(), metric.value);
            },
            MetricType::Timer => {
                let values = self.timers.entry(metric.name.clone()).or_insert(vec![]);
                values.push(metric.value);
            },
            _ => unimplemented!(),
        }
    }

    /// Increase the count of bad messages that could not be parsed.
    #[inline]
    pub fn bad_metric_increase(&mut self) {
        self.bad_metrics += 1;
    }

    /// Return an iterator over the counters.
    pub fn counters_iter(&self) -> hash_map::Iter<Rc<String>, f64> {
        self.counters.iter()
    }

    /// Return an iterator over the gauges.
    pub fn gauges_iter(&self) -> hash_map::Iter<Rc<String>, f64> {
        self.gauges.iter()
    }

    /// Return an iterator over the timer data.
    pub fn timer_data_iter(&self) -> hash_map::Iter<String, f64> {
        self.timer_data.iter()
    }

    /// Clear the counter and timer data.
    pub fn reset(&mut self) {
        self.counters.clear();
        self.timers.clear();
        self.timer_data.clear();
    }

    /// Make timer data statistics.
    pub fn make_timer_stats(&mut self) {
        let mut timer_data: HashMap<String, f64> = HashMap::new();

        for (metric, times) in self.timers.iter_mut() {
            // Sort the metrics for calculating statistics.
            times.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let count = times.len() as f64;
            let sum: f64 = times.iter().sum();
            let average = sum / count;
            let std_dev = get_std_dev(&times, average, count);
            let median = get_percentile(&times, count, 0.5);
            let upper_ninety_five = get_percentile(&times, count, 0.95);

            timer_data.insert(String::from(&*metric.clone().as_str()) + ".min", times[0]);
            timer_data.insert(String::from(&*metric.clone().as_str()) + ".max", times[times.len() - 1]);
            timer_data.insert(String::from(&*metric.clone().as_str()) + ".count", count);
            timer_data.insert(String::from(&*metric.clone().as_str()) + ".average", average);
            timer_data.insert(String::from(&*metric.clone().as_str()) + ".std_dev", std_dev);
            timer_data.insert(String::from(&*metric.clone().as_str()) + ".median", median);
            timer_data.insert(String::from(&*metric.clone().as_str()) + ".upper_95", upper_ninety_five);
        }

        self.timer_data = timer_data;
    }
}

fn get_percentile(values: &[f64], count: f64, percent: f64) -> f64 {
    let index = (count * percent) as usize;
    if values.len() % 2 == 0 {
        return (values[index - 1] + values[index]) / 2.0;
    }
    values[index]
}

fn get_std_dev(values: &[f64], average: f64, count: f64) -> f64 {
    values.iter().map(|v| (v - average).powi(2)).sum::<f64>() / count
}
