//! The graphite module is the default backend for capella.

use std::io::{self, Write};
use std::fmt::Write as FmtWrite;
use std::net::{SocketAddr, ToSocketAddrs};

use chrono::offset::local::Local;

use futures::Future;

use tokio_core::net::TcpStream;
use tokio_core::reactor::Core;

use backend::Backend;

use cache::CapellaCache;

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
}

impl Backend for Graphite {
    // TODO: Clear the cache?
    fn purge_metrics(&self, cache: &mut CapellaCache) {
        let mut core = Core::new().unwrap();
        let handle = core.handle();

        let unix_time = Local::now().timestamp();
        let mut buffer = String::new();

        for (k, v) in cache.counters_iter() {
            write!(buffer, "{} {} {}\n", k, v, unix_time);
        }

        for (k, v) in cache.gauges_iter() {
            write!(buffer, "{} {} {}\n", k, v, unix_time);
        }
        println!("{}", buffer);

        let send = TcpStream::connect(&self.addr, &handle).and_then(|mut out| {
            out.write_all(buffer.as_bytes());
            Ok(())
        });
        drop(core.run(send));
    }
}
