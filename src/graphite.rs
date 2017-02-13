//! The graphite module is the default backend for capella.
#![deny(missing_docs)]

use std::io;
use std::net::{SocketAddr, ToSocketAddrs};

use chrono::offset::local::Local;

use futures::Future;

use tokio_core::net::TcpStream;
use tokio_core::reactor::Core;

use backend::Backend;

use cache::CapellaCache;

const CAPELLA_METRICS_TOTAL: &'static str = "capella.total_metrics";
const CAPELLA_BAD_METRICS_TOTAL: &'static str = "capella.bad_metrics";
const COUNT_SUFFIX: &'static str = ".count";

/// The backend to a graphite server.
#[derive(Debug)]
pub struct Graphite {
    addr: SocketAddr,
}

impl Graphite {
    /// Construct a new graphite instance with a given address.
    pub fn new<A: ToSocketAddrs>(addr: A) -> io::Result<Graphite> {
        Ok(Graphite {
            addr: addr.to_socket_addrs()?.next().unwrap(),
        })
    }

    // Construct a string for the graphite new line API.
    fn make_metric_string(&self, name: &str, value: &f64, time: &str) -> String {
        let mut s = String::new();
        s.push_str(name);
        s.push_str(" ");
        s.push_str(&value.to_string());
        s.push_str(" ");
        s.push_str(time);
        s.push_str("\n");

        s
    }
}

impl Backend for Graphite {
    fn purge_metrics(&self, cache: &mut CapellaCache) {
        let mut core = Core::new().unwrap();
        let handle = core.handle();

        let unix_time = Local::now().timestamp().to_string();
        let mut buffer = String::new();
        cache.make_timer_stats();

        for (k, v) in cache.counters_iter() {
            let metric_str = self.make_metric_string(k, v, &unix_time);
            buffer.push_str(&metric_str);
        }

        for (k, v) in cache.gauges_iter() {
            let metric_str = self.make_metric_string(k, v, &unix_time);
            buffer.push_str(&metric_str);
        }

        for (k, v) in cache.timer_data_iter() {
            let metric_str = self.make_metric_string(k, v, &unix_time);
            buffer.push_str(&metric_str);
        }

        for (k, v) in cache.sets_iter() {
            let key = String::from(&*k.clone().as_str()) + COUNT_SUFFIX;
            let value = v.len() as f64;
            let metric_str = self.make_metric_string(&key, &value, &unix_time);
            buffer.push_str(&metric_str);
        }

        // Add our total message and bad message counts.
        buffer.push_str(&self.make_metric_string(CAPELLA_METRICS_TOTAL, &cache.total_metrics(), &unix_time));
        buffer.push_str(&self.make_metric_string(CAPELLA_BAD_METRICS_TOTAL, &cache.total_bad_metrics(), &unix_time));

        let send = TcpStream::connect(&self.addr, &handle).and_then(|out| {
            ::tokio_core::io::write_all(out, buffer)
        });
        drop(core.run(send));

        cache.reset();
    }
}
