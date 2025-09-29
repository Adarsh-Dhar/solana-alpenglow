# Alpenglow: A Dual-Path, Rotor-Optimized Consensus Protocol for Solana

Author: Alpenglow Team

Version: 1.0

## Abstract

Alpenglow is a next-generation consensus protocol for Solana that achieves 100–150 ms finalization by combining a dual-path BFT mechanism (Votor) with an erasure-coded single-hop block propagation layer (Rotor). We present the protocol design, safety and liveness arguments, formal verification artifacts, and empirical performance characteristics including latency, throughput, and scalability. We further document implementation details and operational guidance for deployment.

## 1. Introduction

Solana’s existing TowerBFT is optimized for throughput but bounded by multi-round finality. Alpenglow introduces two key advances:

1) Votor: a dual-path finality engine that finalizes in a single round when ≥80% stake is responsive (fast path) and in two rounds when ≥60% stake participates (slow path).
2) Rotor: a network dissemination component leveraging erasure coding and stake-weighted sampling to deliver blocks in a single hop with high reliability.

Together, they deliver dramatic latency reductions while maintaining robust safety under Byzantine faults and partial synchrony.

## 2. System Model and Assumptions

- Partial synchrony with eventual message delivery and bounded delays after GST.
- Validator set with stake weights; ≤20% Byzantine stake tolerated; up to an additional 20% may be offline in resilience analysis.
- Cryptographic assumptions: standard signature unforgeability; erasure coding integrity; collision-resistant hashing.

## 3. Protocol Overview

### 3.1 Roles and Data Types

- Leader: proposes a block for a given slot.
- Validators: vote on proposals and aggregate certificates.
- Certificates: aggregated, threshold-signed attestations for proposals or timeouts.
- Skip Certificates: timeouts enabling safe progress under asynchrony or faulty leaders.

### 3.2 Votor Dual-Path Finality

- Fast Path (≥80% stake):
  - One round of voting on the proposal.
  - If certificate reaches threshold, block is finalized.

- Slow Path (≥60% stake):
  - Two rounds with locking to prevent conflicting finalization.
  - Bounded finalization time: min(δ₈₀%, 2·δ₆₀%).

### 3.3 Rotor Propagation

- The leader erasure-codes a block into data and parity shards.
- Validators fetch a minimal subset via stake-weighted relay sampling.
- Single-hop reconstruction with high probability; robust to churn and partial failures.

### 3.4 Timeouts and Recovery

- Skip certificates issued on timeouts to avoid deadlock.
- BadWindow logic reduces oscillations; ensures monotonic progress.

## 4. Safety

Safety requires that no two conflicting blocks finalize for the same slot. Alpenglow enforces safety via:

- Locking rules across rounds to prevent equivocation-based conflicts.
- Threshold certificates with unique aggregation per slot.
- Non-equivocation constraints in validator vote handling.

Formal models (see `alpenglow-formal`) prove:

- Certificate uniqueness per slot.
- No conflicting finalization under ≤20% Byzantine stake.
- Chain consistency across honest validators.

## 5. Liveness

Under partial synchrony and sufficient honest participation:

- Fast Path: completes in a single round when ≥80% stake is responsive.
- Slow Path: completes in two rounds with ≥60% stake responsive.
- Timeouts with skip certificates guarantee recovery from faulty leaders or network hiccups.

These properties are validated via model checking and statistical verification.

## 6. Rotor Analysis

- Single-hop dissemination minimizes latency compared to multi-hop gossip.
- Erasure coding reduces duplicate transmissions while increasing robustness.
- Stake-weighted relay sampling aligns bandwidth usage with validator capacity.

Simulation results (see `alpenglow/latency_histogram.py` and `bandwidth_plot.py`) indicate near-constant latency across a range of validator counts within configured network assumptions.

## 7. Performance Evaluation

### 7.1 Methodology

- Micro-benchmarks via `cargo bench`.
- Simulations leveraging public ping datasets; see `download_data.sh`.
- In-proc cluster (`./run.sh`) with 6 nodes on localhost for functional validation.

### 7.2 Results (Indicative)

- Finalization latency: 100–150 ms under fast-path conditions.
- Throughput: 100k+ TPS potential assuming adequate hardware and network capacity.
- Scalability: Supports 1000+ validators with Rotor’s single-hop scheme.

Assumptions and detailed numbers are provided in [performance.md](./performance.md).

## 8. Implementation Notes

The Rust codebase is divided into consensus, dissemination, networking, and crypto modules. Key binaries:

- `node`: rudimentary standalone node (see [node.md](./node.md)).
- `simulations`: protocol simulations (see [simulations.md](./simulations.md)).
- `performance_test`, `workload_generator`: load and latency tools.

## 9. Formal Verification

We provide machine-checkable models using Stateright with binaries covering safety, liveness, resilience, leader rotation, certificate uniqueness, rotor sampling, and timeouts. Scripts under `alpenglow-formal/scripts/` enable full and quick verification runs. See [verification.md](./verification.md).

## 10. Deployment and Operations

- Configure cluster peers and ports as documented in `node.md`.
- Monitor with `RUST_LOG` and tracing via `fastrace`.
- Use skip certificates and BadWindow tuning in adverse networks.

## 11. Security Considerations

- Validate leader equivocation handling; ensure signature verification and certificate uniqueness checks are enforced.
- Audit erasure coding and shard reconstruction.
- Rate-limit and prioritize traffic to avoid DoS amplification via Rotor.

## 12. Conclusion

Alpenglow delivers substantial latency improvements while preserving safety and robustness through a dual-path BFT core and a single-hop, erasure-coded propagation layer. Formal verification and empirical tests jointly support its readiness for high-performance blockchain systems.

## References

See root `README.md` for literature and resources.

