# Benchmarking capella
This is a relatively simple benchmarking binary that can be used to get a view into how
well capella can handle load from simulated clients. It must be used in the same directory as the
`sample_metrics.txt` file in order to run.

## Usage
```sh
go build bench.go

./bench -h
```

## Output
The suggested use of the benchmark is to use the `console` backend for capella. This will print the
metrics to the console which include the number of metrics seens during the flushing interval.
