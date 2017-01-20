use std::{io, str};
use std::net::SocketAddr;

use futures::{Stream, Sink};

use tokio_core::net::UdpCodec;

/// StatsCodec defines the UDP parser used to accept packets and returns a new
/// statistic or an error.
pub struct StatsCodec;

impl UdpCodec for StatsCodec {
    type In = (SocketAddr, Vec<u8>);
    type Out = SocketAddr;

    fn decode(&mut self, addr: &SocketAddr, buf: &[u8]) -> io::Result<Self::In> {
        Ok((*addr, buf.to_vec()))
    }

    fn encode(&mut self, addr: Self::Out, _: &mut Vec<u8>) -> SocketAddr {
        addr
    }
}
