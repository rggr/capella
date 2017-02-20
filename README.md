# capella
[capella](https://en.wikipedia.org/wiki/Capella) is an aysnchronous StatsD server written in Rust.

![galaxy](galaxy.jpg)

[![Build Status](https://travis-ci.org/rggr/capella.svg?branch=master)](https://travis-ci.org/rggr/capella)

## Documentation
- [Building and Testing](#building-and-testing)
- [Configuration](#configuration)
- [Supported Metrics](#supported-metrics)
- [Future Plans](#future-plans)

## Building and Testing
capella uses [Cargo](https://github.com/rust-lang/cargo) to build and test. You can also install
it directly from the [Crates](https://crates.io) website.

```sh
# Install from crates.io.
cargo install capella

# Building capella with optimizations.
cargo build --release

# Running the unit tests.
cargo test
```

## Configuration
capella uses an environment variable based configuration file named `capella.env`. capella expects
this file to be in the same directory as the binary. Currently the necessary configuration values
needed are as follows:

```sh
# The connection string for the graphite host. It includes an IP address as well as a port.
CAPELLA_GRAPHITE_CONNECTION=127.0.0.1:2003

# The address and port on which capella should listen.
CAPELLA_LISTENER=127.0.0.1:8125

# The flushing duration defines how long capella buffers metrics before sending to graphite.
# It is defined in seconds.
CAPELLA_FLUSH_DURATION=10

# Set the log level for the `env_logger` module.
RUST_LOG=info
```

## Supported Metrics
capella supports the four metrics that StatsD implements. They are counter, gauges, timers, and sets.

#### Counters
Counters represent metrics that can only increase. Counters can also have an associated sampling
rate that tells capella that a metric is only being sent for a fraction of the time.

```sh
# This tells capella that the rate is only being sent once every half of the flush duration.
counter:1|c|@0.5
```

#### Gauges
Gauges are metrics that can fluctuate both negatively and postively. They are similar to a gauge
in a car.

```sh
# This subtracts one from the current value for the "gauge" key.
gauge:-1|g
```

#### Sets
Sets are metrics that hold a unique collection of values. The metric derived by capella for this
type is the cardinality of the set.

```sh
# Add a new value to the set which is only added if it doesn't exist.
set:11|s
```

#### Timers
Timers are unique in that many statistics are derived from them. Per flush duration, timers generate
the following:
- Minimum value
- Maximum value
- Count
- Average
- Standard Deviation
- Median
- 95th Percentile

Timers also support sampling.

```sh
timer:1.5|ms
```

## Future Plans
Currently capella is not nearly as configurable as the original StatsD. It may never be but
support for the most used options will be added on an as-needed basis. capella will continue to add
unit tests to increase code coverage and ensure that metrics handling is identical to StatsD.
Please open issues to ask questions about adding features or to point out bugs.

### Authors
capella was started by [Garrett](https://github.com/gsquire) and [Ralph](https://github.com/deckarep).
The name is inspired from one of Garrett's favorite bands, Rosetta.

### License
capella is released under the MIT license.
