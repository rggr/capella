# capella
capella is an aysnchronous StatsD server written in Rust.

[![Build Status](https://travis-ci.org/rggr/capella.svg?branch=master)](https://travis-ci.org/rggr/capella)

## Documentation
- [Supported Metrics](#supported-metrics)

## Supported Metrics
capella supports the four metrics that StatsD implements. They are counter, gauges, timers, and sets.

### Building and Testing
capella uses [Cargo](https://github.com/rust-lang/cargo) to build and test.

```sh
# Building capella with optimizations.
cargo build --release

# Running the unit tests.
cargo test
```

### Authors
capella was started by [Garrett](https://github.com/gsquire) and [Ralph](https://github.com/deckarep).
The name is inspired from one of Garrett's favorite bands, Rosetta.

### License
capella is released under the MIT license.
