//! The cache module dictates how metrics will be buffered before being sent to the
//! corresponding backend.

use std::collections::HashMap;

use parse::{Metric, MetricType};

/// `CapellaCache` is the bucketing mechanism used by capella to buffer metrics before sending to
/// the backend.
// TODO: Can we do better than storing owned strings?
#[derive(Debug, Default)]
pub struct CapellaCache {
    counters: HashMap<String, i64>,
    gauges: HashMap<String, i64>,
    metrics_seen: u64,
    bad_metrics: u64,
}

impl CapellaCache {
    /// This function will add a `Metric` to the cache.
    pub fn add_metric(&mut self, metric: &Metric) {
        self.metrics_seen += 1;

        match metric.metric_type {
            MetricType::Counter => {
                let c = self.counters.entry(metric.name.clone()).or_insert(0);
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
}
