# Standalone Node

Rudimentary standalone node for functional testing.

## Define the Cluster

Prepare a file with base ports, one per line:

```csv
127.0.0.1:3000
127.0.0.1:3003
127.0.0.1:3006
127.0.0.1:3009
```

## Generate Configs

```bash
cargo run --release --bin node -- --generate-config-files ip_list --config-name ag_node
```

## Run Nodes

```bash
cargo run --release --bin node -- --config-name ag_node_0.toml &
cargo run --release --bin node -- --config-name ag_node_1.toml &
cargo run --release --bin node -- --config-name ag_node_2.toml &
cargo run --release --bin node -- --config-name ag_node_3.toml
```

You can stop and restart a node without halting the cluster.

