#[macro_use]
extern crate lazy_static;

extern crate futures;
extern crate regex;
extern crate tokio_core;

mod error;
mod parse;
mod server;

use server::start_udp_server;

fn main() {
    println!("starting server...");
    start_udp_server();
}
