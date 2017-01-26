#[macro_use]
extern crate lazy_static;

extern crate futures;
extern crate regex;
extern crate tokio_core;

mod cache;
mod error;
mod parse;
mod server;

use cache::CapellaCache;

use server::start_udp_server;

fn main() {
    println!("starting server...");

    let mut cache = CapellaCache::default();
    start_udp_server(&mut cache);
}
