# Performance & Benchmarks

## Assumptions

- Partial synchrony with bounded delays after GST.
- Network RTT distributions derived from public ping dataset (see `download_data.sh`).
- Validator hardware provisioned for 100k+ TPS class workloads.

## Latency

- Fast path: 100–150 ms finalization when ≥80% stake is responsive.
- Slow path: two-round finalization with ≥60% stake participation.

## Throughput & Scalability

- Target: 100k+ TPS with Rotor minimizing propagation overhead.
- Scales to 1000+ validators with single-hop dissemination.

## How to Reproduce

```bash
cargo bench
RUST_LOG="simulations=debug" ./download_data.sh && cargo run --release --bin simulations
./run.sh # local 6-node cluster
```

See `alpenglow/latency_histogram.py` and `bandwidth_plot.py` for analysis.

