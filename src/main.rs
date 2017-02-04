#[macro_use]
extern crate lazy_static;

extern crate chrono;
extern crate futures;
extern crate regex;
extern crate tokio_core;
extern crate tokio_timer;

mod backend;
mod cache;
mod console;
mod error;
mod graphite;
mod parse;
mod server;

use graphite::Graphite;

use server::start_udp_server;

fn main() {
    println!("starting server...");

    let backend = Graphite::new("127.0.0.1:2003").unwrap();
    start_udp_server(backend);
}
