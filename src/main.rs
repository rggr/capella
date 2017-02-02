#[macro_use]
extern crate lazy_static;

extern crate futures;
extern crate regex;
extern crate tokio_core;
extern crate tokio_timer;

mod backend;
mod cache;
mod console;
mod error;
mod parse;
mod server;

use console::Console;

use server::start_udp_server;

fn main() {
    println!("starting server...");

    let backend = Console::default();
    start_udp_server(backend);
}
