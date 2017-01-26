//! The server module defines the codec used for parsing stats in capella.

use std::io;
use std::net::SocketAddr;

use futures::{Stream, Sink};

use tokio_core::net::{UdpCodec, UdpSocket};
use tokio_core::reactor::Core;

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
pub fn start_udp_server(cache: &mut CapellaCache) {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let addr: SocketAddr = "127.0.0.1:8125".parse().unwrap();
    let s = UdpSocket::bind(&addr, &handle).unwrap();

    let (sink, stream) = s.framed(StatsCodec).split();

    // This is the event loop stream in which all values are parsed.
    // TODO: Investigate why I can't figure out how to chain functions to catch the error case.
    use error::CapellaResult;
    let events = stream.then(|res| {
        let v = res.unwrap_or(("0.0.0.0:8125".parse().unwrap(), vec![]));
        if v.1.len() == 0 {
            cache.bad_metric_increase();
        }
        println!("{:?}", v.1);
        for m in &v.1 {
            cache.add_metric(m);
        }
        let r: CapellaResult<SocketAddr> = Ok(v.0);
        r
    });
    let f = sink.send_all(events);

    drop(core.run(f));
}
