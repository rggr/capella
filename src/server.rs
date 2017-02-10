//! The server module defines the codec used for parsing stats in capella.

use std::io;
use std::env;
use std::cell::RefCell;
use std::error::Error;
use std::net::SocketAddr;
use std::rc::Rc;
use std::time::Duration;

use futures::{Future, Stream};

use tokio_core::net::{UdpCodec, UdpSocket};
use tokio_core::reactor::Core;

use tokio_timer::Timer;

use backend::Backend;

use cache::CapellaCache;

use parse::{self, Metric};

/// `StatsCodec` defines the UDP parser used to accept packets and returns a new
/// statistic or an error.
pub struct StatsCodec;

impl UdpCodec for StatsCodec {
    type In = (SocketAddr, Vec<Metric>);
    type Out = SocketAddr;

    fn decode(&mut self, addr: &SocketAddr, buf: &[u8]) -> io::Result<Self::In> {
        let mut metrics = Vec::new();

        // See if we got multiple metrics.
        if buf.contains(&b'\n') {
            // Based on the behavior of split, we need to filter out zero-length chunks.
            for m in buf.split(|c| *c == b'\n').filter(|chunk| chunk.len() > 0) {
                let metric = parse::parse_metric(m);
                if metric.is_ok() {
                    metrics.push(metric.unwrap());
                }
            }
            return Ok((*addr, metrics));
        }

        // We only got one metric sent.
        let metric = parse::parse_metric(buf);
        if metric.is_ok() {
            metrics.push(metric.unwrap());
        }

        Ok((*addr, metrics))
    }

    // Since stat collecting is fire and forget, we don't need to write data
    // back to the client.
    fn encode(&mut self, addr: Self::Out, _: &mut Vec<u8>) -> SocketAddr {
        addr
    }
}

/// This starts up the UDP server with the default backend being a graphite host.
/// Other backends can be specified by modifying the main program.
pub fn start_udp_server<B: Backend>(backend: B) {
    let cache = Rc::new(RefCell::new(CapellaCache::default()));
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let capella_addr = env::var("CAPELLA_LISTENER").unwrap();
    let addr: SocketAddr = capella_addr.parse().unwrap();
    let s = UdpSocket::bind(&addr, &handle).unwrap();

    let (_, stream) = s.framed(StatsCodec).split();

    // This sets up the purge timer utilizing the event loop.
    let flush_duration = env::var("CAPELLA_FLUSH_DURATION").unwrap().parse::<u64>().unwrap();
    let timer = Timer::default().interval(Duration::new(flush_duration, 0));
    let future_t = timer.for_each(|()| {
        backend.purge_metrics(&mut cache.borrow_mut());
        Ok(())
    }).map_err(|e| {
        io::Error::new(io::ErrorKind::Other, e.description())
    });

    // This is the event loop stream in which all values are parsed.
    let events = stream.for_each(|(_, metrics)| {
        if metrics.is_empty() {
            cache.borrow_mut().bad_metric_count_increase();
        }

        for m in &metrics {
            cache.borrow_mut().add_metric(m);
        }

        Ok(())
    });
    let f = events.join(future_t);

    drop(core.run(f));
}
