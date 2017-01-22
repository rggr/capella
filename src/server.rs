use std::io;
use std::net::SocketAddr;

use futures::{Stream, Sink};

use tokio_core::net::UdpCodec;

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
            for m in buf.split(|c| *c == b'\n') {
                let metric = parse::parse_metric(m)?;
                metrics.push(metric);
            }
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
