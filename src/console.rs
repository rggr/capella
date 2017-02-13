//! The console module is used mostly for testing purposes.
#![deny(missing_docs)]

use backend::Backend;

use cache::CapellaCache;

/// Console is a unit struct that prints stats to the terminal.
#[derive(Default)]
pub struct Console;

impl Backend for Console {
    fn purge_metrics(&self, cache: &mut CapellaCache) {
        println!("{:?}", cache);
    }
}
