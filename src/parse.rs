//! The parse module is responsible for parsing events published by clients.

use regex::Regex;

use std::str::{self, FromStr};

use error::{Error, CapellaResult};

/// MetricType defines what kind of metric was parsed from a client.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Meter,
    Timer,
}

impl FromStr for MetricType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "c" => Ok(MetricType::Counter),
            "g" => Ok(MetricType::Gauge),
            "h" => Ok(MetricType::Histogram),
            "m" => Ok(MetricType::Meter),
            "ms" => Ok(MetricType::Timer),
            _ => Err(Error::Parse),
        }
    }
}

/// A Metric defines a published client event.
#[derive(Debug, PartialEq)]
pub struct Metric {
    pub name: String,
    pub value: i64,
    pub metric_type: MetricType,
    pub sample_rate: Option<f32>,
}

impl Metric {
    pub fn new() -> Metric {
        Metric {
            name: String::new(),
            value: 0,
            metric_type: MetricType::Meter,
            sample_rate: None,
        }
    }
}

/// The parse_metric function trys to break down a single UDP packet into a single metric.
pub fn parse_metric(packet: &[u8]) -> CapellaResult<Metric> {
    lazy_static! {
        // TODO: Learn how to write this over multiple lines.
        static ref PATTERN: Regex = Regex::new(r"(?P<name>[\w\.]+):((?P<sign>\-|\+))?(?P<val>\d+)\|(?P<type>\w+)(\|@(?P<rate>\d+\.\d+))?").unwrap();
    }

    if let Ok(val) = str::from_utf8(packet) {
        if !PATTERN.is_match(val) {
            return Err(Error::Parse);
        }
    } else {
        return Err(Error::Parse);
    }

    // We know this matched before so it is safe to unwrap.
    let caps = PATTERN.captures(str::from_utf8(packet).unwrap()).unwrap();

    let mut metric = Metric::new();
    // These are required to match.
    let name = caps.name("name").unwrap().as_str();
    let value = caps.name("val").unwrap().as_str().parse::<i64>().map_err(Error::from)?;
    let metric_type = caps.name("type").unwrap().as_str();

    metric.name = String::from(name);
    metric.value = value;
    metric.metric_type = metric_type.parse::<MetricType>()?;

    // Now see if there were optional values added in.
    if let Some(sign) = caps.name("sign") {
        let s = sign.as_str();
        if s == "-" {
            metric.value *= -1;
        }
    }

    if let Some(rate) = caps.name("rate") {
        let r = rate.as_str().parse::<f32>().map_err(Error::from)?;
        metric.sample_rate = Some(r);
    }

    Ok(metric)
}

#[cfg(test)]
mod tests {
    use super::{Metric, MetricType, parse_metric};

    #[test]
    fn good_simple_meter() {
        let packet = b"test:1|m";
        let m1 = parse_metric(packet).unwrap();

        let mut m2 = Metric::new();
        m2.name = String::from("test");
        m2.value = 1;

        assert_eq!(m1, m2);
    }

    #[test]
    fn bad_parse_double_colon() {
        let packet = b"test::1|c";
        assert!(parse_metric(packet).is_err());
    }

    #[test]
    fn good_timing_with_rate() {
        let packet = b"test:1|ms|@0.1";
        let m1 = parse_metric(packet).unwrap();

        let mut m2 = Metric::new();
        m2.name = String::from("test");
        m2.value = 1;
        m2.metric_type = MetricType::Timer;
        m2.sample_rate = Some(0.1);

        assert_eq!(m1, m2);
    }

    #[test]
    fn bad_parse_float_value() {
        let packet = b"test:1.0|g";
        assert!(parse_metric(packet).is_err());
    }

    #[test]
    fn bad_parse_unknown_metric_type() {
        let packet = b"test:1|a";
        assert!(parse_metric(packet).is_err());
    }

    #[test]
    fn good_nested_metric_name() {
        let packet = b"test.nested.name:1|c";
        let m1 = parse_metric(packet).unwrap();

        let mut m2 = Metric::new();
        m2.name = String::from("test.nested.name");
        m2.value = 1;
        m2.metric_type = MetricType::Counter;

        assert_eq!(m1, m2);
    }
}
