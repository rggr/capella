#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

extern crate chrono;
extern crate dotenv;
extern crate env_logger;
extern crate futures;
extern crate regex;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_timer;

pub mod backend;
pub mod cache;
pub mod console;
pub mod error;
pub mod graphite;
pub mod parse;
pub mod server;

use std::env;

use graphite::Graphite;

use server::start_udp_server;

// Print the current environment information.
fn print_setup() {
    info!("current capella environment:");
    for (k, v) in env::vars() {
        if k.starts_with("CAPELLA") {
            info!("{} = {}", k, v);
        }
    }
}

fn main() {
    // Setup our environment.
    dotenv::from_filename("capella.env").ok();
    env_logger::init().unwrap();

    let graphite_conn = env::var("CAPELLA_GRAPHITE_CONNECTION").unwrap();
    let backend = Graphite::new(graphite_conn.as_str()).unwrap();

    print_setup();

    start_udp_server(backend);
}
