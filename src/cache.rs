//! The cache module dictates how metrics will be buffered before being sent to the
//! corresponding backend.

use std::collections::{hash_map, HashMap, HashSet};
use std::rc::Rc;

use parse::{Metric, MetricType};

/// `CapellaCache` is the bucketing mechanism used by capella to buffer metrics before sending to
/// the backend.
#[derive(Debug, Default)]
pub struct CapellaCache {
    counters: HashMap<Rc<String>, f64>,
    gauges: HashMap<Rc<String>, f64>,
    timers: HashMap<Rc<String>, Vec<f64>>,
    sets: HashMap<Rc<String>, HashSet<i64>>,
    timer_data: HashMap<String, f64>,
    metrics_seen: u64,
    bad_metrics: u64,
}

impl CapellaCache {
    /// This function will add a `Metric` to the cache.
    pub fn add_metric(&mut self, metric: &Metric) {
        self.metric_count_increase();

        match metric.metric_type {
            MetricType::Counter => {
                let c = self.counters.entry(metric.name.clone()).or_insert(0.0);
                *c += metric.value * metric.sample_rate.unwrap_or(1.0);
            },
            MetricType::Gauge => {
                self.gauges.insert(metric.name.clone(), metric.value);
            },
            MetricType::Timer => {
                let values = self.timers.entry(metric.name.clone()).or_insert(Vec::new());
                values.push(metric.value * metric.sample_rate.unwrap_or(1.0));
            },
            MetricType::Set => {
                let values = self.sets.entry(metric.name.clone()).or_insert(HashSet::new());
                values.insert(metric.value.round() as i64);
            }
        }
    }

    /// Increase the count of bad messages that could not be parsed.
    #[inline]
    pub fn bad_metric_count_increase(&mut self) {
        self.bad_metrics += 1;
        self.metric_count_increase();
    }

    /// Increase the unique metric count.
    #[inline]
    pub fn metric_count_increase(&mut self) {
        self.metrics_seen += 1;
    }

    /// Return the total number of metrics seen.
    #[inline]
    pub fn total_metrics(&self) -> f64 {
        self.metrics_seen as f64
    }

    /// Return the total number of failed metrics seen.
    #[inline]
    pub fn total_bad_metrics(&self) -> f64 {
        self.bad_metrics as f64
    }

    /// Return an iterator over the counters.
    pub fn counters_iter(&self) -> hash_map::Iter<Rc<String>, f64> {
        self.counters.iter()
    }

    /// Return an iterator over the gauges.
    pub fn gauges_iter(&self) -> hash_map::Iter<Rc<String>, f64> {
        self.gauges.iter()
    }

    /// Return an iterator over the sets.
    pub fn sets_iter(&self) -> hash_map::Iter<Rc<String>, HashSet<i64>> {
        self.sets.iter()
    }

    /// Return an iterator over the timer data.
    pub fn timer_data_iter(&self) -> hash_map::Iter<String, f64> {
        self.timer_data.iter()
    }

    /// Clear the counter and timer data.
    pub fn reset(&mut self) {
        self.counters.clear();
        self.sets.clear();
        self.timers.clear();
        self.timer_data.clear();
    }

    /// Make timer data statistics.
    pub fn make_timer_stats(&mut self) {
        let mut timer_data: HashMap<String, f64> = HashMap::new();

        for (metric, times) in &mut self.timers {
            // Sort the metrics for calculating statistics.
            times.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let count = times.len() as f64;
            let sum: f64 = times.iter().sum();
            let average = sum / count;
            let std_dev = get_std_dev(times, average, count);
            let median = get_percentile(times, count, 0.5);
            let upper_ninety_five = get_percentile(times, count, 0.95);

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
    (values.iter().map(|v| (v - average).powi(2)).sum::<f64>() / count).sqrt()
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::CapellaCache;
    use parse::{Metric, MetricType};

    const EPSILON: f64 = 1e-32;

    // Create a new timer metric.
    fn make_timer_metric(name: &str, value: f64) -> Metric {
        Metric {
            name: Rc::new(String::from(name)),
            value: value,
            metric_type: MetricType::Timer,
            sample_rate: None,
        }
    }

    #[test]
    fn timer_data_generation() {
        let mut timers = Vec::new();
        let mut cache = CapellaCache::default();
        let two: f64 = 2.0;
        let sqrt_two = two.sqrt();

        timers.push(make_timer_metric("test", 1.0));
        timers.push(make_timer_metric("test", 2.0));
        timers.push(make_timer_metric("test", 3.0));
        timers.push(make_timer_metric("test", 4.0));
        timers.push(make_timer_metric("test", 5.0));

        for t in &timers {
            cache.add_metric(t);
        }
        cache.make_timer_stats();

        assert!((cache.timer_data.get("test.min").unwrap() - 1.0).abs() < EPSILON);
        assert!((cache.timer_data.get("test.max").unwrap() - 5.0).abs() < EPSILON);
        assert!((cache.timer_data.get("test.count").unwrap() - 5.0).abs() < EPSILON);
        assert!((cache.timer_data.get("test.average").unwrap() - 3.0).abs() < EPSILON);
        assert!((cache.timer_data.get("test.std_dev").unwrap() - sqrt_two).abs() < EPSILON);
        assert!((cache.timer_data.get("test.median").unwrap() - 3.0).abs() < EPSILON);
        assert!((cache.timer_data.get("test.upper_95").unwrap() - 5.0).abs() < EPSILON);
    }
}
