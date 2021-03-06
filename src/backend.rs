//! The backend module defines the trait that a backend must implement in order
//! to be forwarded stats from capella.
#![deny(missing_docs)]

use cache::CapellaCache;

/// Backend defines a generic backend that can be forwarded metrics from capella.
pub trait Backend {
    /// Flush metrics accepts a `CapellaCache` type and forwards it to the backend that
    /// implements it.
    fn purge_metrics(&self, cache: &mut CapellaCache);
}
