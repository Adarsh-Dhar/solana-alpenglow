# Architecture

This document describes the components of Alpenglow and their interactions.

## Components

- Votor (consensus engine): fast/slow path voting, locking, certificates.
- Rotor (propagation): erasure coding, relay sampling, reconstruction.
- Timeout/Skip Certificates: recovery and progress.
- Networking: UDP local simulator and TCP/UDP abstractions.
- Crypto: hashing, signatures, Reed-Solomon erasure coding.

## Data Flow

1. Leader proposes block → erasure-code via Rotor → shards disseminated.
2. Validators reconstruct block → validate → vote.
3. Votes aggregated → certificate → finalize or move to next round.
4. On timeout → skip certificate → advance safely.

## Source Layout

Refer to `alpenglow/src` and `alpenglow-formal/src` for implementation and models.

