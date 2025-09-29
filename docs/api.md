# API & Crate Layout

Generate Rust API docs:

```bash
cargo doc --no-deps --open
```

## Crate Structure

- `alpenglow/src` — core consensus, rotor, networking, crypto, types.
- `alpenglow/src/bin` — binaries: `node`, `simulations`, `performance_test`, `workload_generator`.
- `alpenglow-formal/src` — formal models and verification binaries.

Refer to inline Rust docs for module-level details.

