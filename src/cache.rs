//! The cache module dictates how metrics will be buffered before being sent to the
//! corresponding backend.

use std::collections::{hash_map, HashMap};

use parse::{Metric, MetricType};

/// `CapellaCache` is the bucketing mechanism used by capella to buffer metrics before sending to
/// the backend.
// TODO: Can we do better than storing owned strings?
#[derive(Debug, Default)]
pub struct CapellaCache {
    counters: HashMap<String, f64>,
    gauges: HashMap<String, f64>,
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
            _ => unimplemented!(),
        }
    }

    /// Increase the count of bad messages that could not be parsed.
    #[inline]
    pub fn bad_metric_increase(&mut self) {
        self.bad_metrics += 1;
    }

    /// Return an iterator over the counters.
    pub fn counters_iter(&self) -> hash_map::Iter<String, f64> {
        self.counters.iter()
    }

    /// Return an iterator over the gauges.
    pub fn gauges_iter(&self) -> hash_map::Iter<String, f64> {
        self.gauges.iter()
    }

    /// Clear the counter and timer data.
    pub fn reset(&mut self) {
        self.counters.clear();
    }
}
