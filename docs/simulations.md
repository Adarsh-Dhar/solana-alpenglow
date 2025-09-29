# Simulations

Protocol-level simulations evaluate Rotor resilience and end-to-end latency/bandwidth.

## Data Preparation

```bash
./download_data.sh
```

## Running

```bash
RUST_LOG="simulations=debug" cargo run --release --bin simulations
```

Configuration is currently via `const` values in `src/bin/simulations/main.rs`.

## Analysis

See `latency_histogram.py` and `bandwidth_plot.py` for visualization.

