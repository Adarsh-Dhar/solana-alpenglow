# Formal Verification

This document outlines the formal verification suite found in `alpenglow-formal/`.

## Contents

- Binaries covering safety, liveness, resilience, leader rotation, certificate uniqueness, rotor sampling, and timeouts.
- Scripts for quick runs, full verification, benchmarking, and reproducibility.

## How to Run

```bash
cd alpenglow-formal
./scripts/quick_test.sh
./scripts/run_all_verifications.sh
```

Run individual models using `cargo run --bin <name> -- --validators N --slots M`.

## Results

Generated results and logs are stored under `alpenglow-formal/results/` with timestamped directories and CSV summaries.

