//! The server module defines the codec used for parsing stats in capella.

use std::io;
use std::cell::RefCell;
use std::error::Error;
use std::net::SocketAddr;
use std::rc::Rc;
use std::time::Duration;

use futures::{Future, Stream};

use tokio_core::net::{UdpCodec, UdpSocket};
use tokio_core::reactor::Core;

use tokio_timer::Timer;

use cache::CapellaCache;

use parse::{self, Metric};

/// StatsCodec defines the UDP parser used to accept packets and returns a new
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
                let metric = parse::parse_metric(m)?;
                metrics.push(metric);
            }
            return Ok((*addr, metrics));
        }

        // We only got one metric sent.
        let metric = parse::parse_metric(buf)?;
        metrics.push(metric);

        Ok((*addr, metrics))
    }

    // Since stat collecting is fire and forget, we don't need to write data
    // back to the client.
    fn encode(&mut self, addr: Self::Out, _: &mut Vec<u8>) -> SocketAddr {
        addr
    }
}

// TODO: This will need to allow for configuration.
pub fn start_udp_server() {
    let cache = Rc::new(RefCell::new(CapellaCache::default()));
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let addr: SocketAddr = "127.0.0.1:8125".parse().unwrap();
    let s = UdpSocket::bind(&addr, &handle).unwrap();

    let (_, stream) = s.framed(StatsCodec).split();

    // This sets up the purge timer utilizing the event loop.
    let t = Timer::default().interval(Duration::new(5, 0));
    let future_t = t.for_each(|()| {
        cache.borrow_mut().purge_metrics();
        Ok(())
    }).map_err(|e| {
        io::Error::new(io::ErrorKind::Other, e.description())
    });

    // This is the event loop stream in which all values are parsed.
    let events = stream.for_each(|(_, metrics)| {
        if metrics.len() == 0 {
            cache.borrow_mut().bad_metric_increase();
        }

        for m in &metrics {
            cache.borrow_mut().add_metric(m);
        }

        Ok(())
    }).map_err(|e| {
        io::Error::new(io::ErrorKind::Other, e.description())
    });
    let f = events.join(future_t);

    drop(core.run(f));
}
