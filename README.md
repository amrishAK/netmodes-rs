# netmodes-rs

`netmodes-rs` is a Rust networking crate focused on TCP server/client workflows with compile-time lifecycle safety.

## Current Scope

- Implemented: TCP server and TCP client handlers with type-state transitions.
- Implemented: callback-driven message handling and client registry context.
- Implemented: cross-platform socket layer for Unix and Windows targets.
- Not yet implemented: UDP handler API (current module is a placeholder).

## Install

Add this to `Cargo.toml`:

```toml
[dependencies]
netmodes-rs = "0.1.0"
```

Import from Rust code:

```rust
use netmodes_rs::tcp_socket_handler::*;
```

## Quick Start

Use the shortest end-to-end path in [doc/getting-started.md](doc/getting-started.md).

If you are working inside this repository, you can run the sample server binary:

```bash
cargo run --bin server
```

## Documentation

- [doc/getting-started.md](doc/getting-started.md): first TCP round-trip (server + client).
- [doc/package-usage.md](doc/package-usage.md): practical usage patterns and lifecycle rules.
- [doc/api-overview.md](doc/api-overview.md): public API surface summary.
- [doc/architecture.md](doc/architecture.md): module boundaries and runtime flow.
- [doc/error-handling.md](doc/error-handling.md): error model and handling strategies.

## Testing

Run all tests:

```bash
cargo test
```

Run unit tests and binary tests:

```bash
cargo test-unit
```

Run integration tests:

```bash
cargo test-integration
```
