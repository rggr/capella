//! The parse module is responsible for parsing events published by clients.
#![deny(missing_docs)]

use regex::Regex;

use std::rc::Rc;
use std::str::{self, FromStr};

use error::{Error, CapellaResult};

/// `MetricType` defines what kind of metric was parsed from a client.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MetricType {
    Counter,
    Gauge,
    Set,
    Timer,
}

impl FromStr for MetricType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "c" => Ok(MetricType::Counter),
            "g" => Ok(MetricType::Gauge),
            "ms" => Ok(MetricType::Timer),
            "s" => Ok(MetricType::Set),
            _ => Err(Error::Parse),
        }
    }
}

/// A `Metric` defines a published client event.
#[derive(Debug, PartialEq)]
pub struct Metric {
    pub name: Rc<String>,
    pub value: f64,
    pub metric_type: MetricType,
    pub sample_rate: Option<f64>,
}

impl Metric {
    pub fn new() -> Metric {
        Metric {
            name: Rc::new(String::new()),
            value: 0.0,
            metric_type: MetricType::Counter,
            sample_rate: None,
        }
    }
}

/// The `parse_metric` function trys to break down a single UDP packet into a single metric.
pub fn parse_metric(packet: &[u8]) -> CapellaResult<Metric> {
    lazy_static! {
        static ref PATTERN: Regex = Regex::new(r"(?x)
            \A(?P<name>[\w\.]+):
            ((?P<sign>\-|\+))?
            (?P<val>([0-9]*[.])?[0-9]+)
            \|(?P<type>\w+)
            (\|@(?P<rate>\d+\.\d+))?\z").unwrap();
    }

    if let Ok(val) = str::from_utf8(packet) {
        if !PATTERN.is_match(val) {
            trace!("UDP packet did not match the pattern");
            return Err(Error::Parse);
        }
    } else {
        trace!("UDP packet was invalid UTF-8");
        return Err(Error::Parse);
    }

    // We know this matched before so it is safe to unwrap.
    let caps = PATTERN.captures(str::from_utf8(packet).unwrap()).unwrap();

    let mut metric = Metric::new();
    // These are required to match.
    let name = caps.name("name").unwrap().as_str();
    let value = caps.name("val").unwrap().as_str().parse::<f64>().map_err(Error::from)?;
    let metric_type = caps.name("type").unwrap().as_str();

    metric.name = Rc::new(String::from(name));
    metric.value = value;
    metric.metric_type = metric_type.parse::<MetricType>()?;

    // Now see if there were optional values added in.
    // Counters cannot be decremented, so only do so if the metric is not a counter.
    if let Some(sign) = caps.name("sign") {
        let s = sign.as_str();
        if s == "-" && metric.metric_type != MetricType::Counter {
            metric.value *= -1.0;
        }
    }

    if let Some(rate) = caps.name("rate") {
        let r = rate.as_str().parse::<f64>().map_err(Error::from)?;
        metric.sample_rate = Some(r);
    }

    Ok(metric)
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::{Metric, MetricType, parse_metric};

    #[test]
    fn bad_parse_cases() {
        let cases = vec!["test::1|c",
                         "",
                         "test|1:",
                         "test:1|c@1",
                         ":1.0|c",
                         "test|1",
                         "test:1|a",
                         "test:c|c",
                         "test:1|ms|0.3"];
        for c in &cases {
            assert!(parse_metric(c.as_bytes()).is_err());
        }
    }

    #[test]
    fn good_simple_counter() {
        let packet = b"test:1|c";
        let m1 = parse_metric(packet).unwrap();

        let mut m2 = Metric::new();
        m2.name = Rc::new(String::from("test"));
        m2.value = 1.0;

        assert_eq!(m1, m2);
    }

    #[test]
    fn good_timing_with_rate() {
        let packet = b"test:1|ms|@0.1";
        let m1 = parse_metric(packet).unwrap();

        let mut m2 = Metric::new();
        m2.name = Rc::new(String::from("test"));
        m2.value = 1.0;
        m2.metric_type = MetricType::Timer;
        m2.sample_rate = Some(0.1);

        assert_eq!(m1, m2);
    }

    #[test]
    fn good_parse_float_value() {
        let packet = b"test:1.0|g";
        assert!(parse_metric(packet).is_ok());
    }

    #[test]
    fn good_nested_metric_name() {
        let packet = b"test.nested.name:1|c";
        let m1 = parse_metric(packet).unwrap();

        let mut m2 = Metric::new();
        m2.name = Rc::new(String::from("test.nested.name"));
        m2.value = 1.0;
        m2.metric_type = MetricType::Counter;

        assert_eq!(m1, m2);
    }
}
