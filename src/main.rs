#[macro_use]
extern crate lazy_static;

extern crate chrono;
extern crate dotenv;
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

use std::env;

use graphite::Graphite;

use server::start_udp_server;

// Print the current environment information.
fn print_setup() {
    println!("current capella environment:");
    for (k, v) in env::vars() {
        if k.starts_with("CAPELLA") {
            println!("\t{} = {}", k, v);
        }
    }
}

fn main() {
    // Setup our environment.
    dotenv::from_filename("capella.env").ok();

    let graphite_conn = env::var("CAPELLA_GRAPHITE_CONNECTION").unwrap();
    let backend = Graphite::new(graphite_conn.as_str()).unwrap();

    print_setup();

    start_udp_server(backend);
}
