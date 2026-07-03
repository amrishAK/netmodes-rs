# thugal-net

`thugal-net` is a Rust networking crate for transport-layer (L4) server/client flows.

It focuses on:

- Strong lifecycle flow with typestate transitions.
- Explicit connection handling.
- Callback-based message handling.
- Shared TCP socket abstractions for Unix and Windows.

The current stable path is TCP. The end-goal target for this project is documented in [docs/requirement.md](docs/requirement.md): a layered transport framework with a poller-based core and protocol extension points.

## Current Scope

Available now:

- TCP server and TCP client handler APIs with typestate lifecycle transitions.
- Message callback flow through `OnMessageHandler`, `TcpClientSession`, and `ContextHandler`.
- Registry-backed context helpers for routing and broadcast-style messaging.
- Cross-platform TCP socket abstractions (Unix and Windows).

In progress:

- Poller-driven runtime foundation (`epoll`/`kqueue`/`WSAPoll`).
- Expanded lifecycle hooks (`on_connect`, `on_message`, `on_disconnect`).
- Protocol codec/adapter extension points for custom L7 framing.

Not yet in stable public API:

- Production-ready UDP server/client handler APIs.
- First-class session registry API from the requirements document.

## Install

Add to `Cargo.toml`:

```toml
[dependencies]
thugal-net = "0.1.0"
```

Use in code:

```rust
use thugal_net::tcp_socket_handler::*;
```

If you prefer `thugal::net` style imports, alias the package:

```toml
[dependencies]
thugal = { package = "thugal-net", version = "0.1.0" }
```

```rust
use thugal::net::tcp_socket_handler::*;
```

## Quick Start

Follow [doc/getting-started.md](doc/getting-started.md) for a minimal TCP round-trip example.

Inside this repository, run the sample server:

```bash
cargo run -p server-module
```

The sample server package is in `example/server_module`.

## Documentation

- [docs/requirement.md](docs/requirement.md): roadmap, requirements, and phased scope.
- [doc/getting-started.md](doc/getting-started.md): first TCP server/client round-trip.
- [doc/package-usage.md](doc/package-usage.md): usage patterns and lifecycle rules.
- [doc/api-overview.md](doc/api-overview.md): public API summary.
- [doc/architecture.md](doc/architecture.md): module boundaries and runtime flow.
- [doc/error-handling.md](doc/error-handling.md): error model and handling approach.

## Testing

Run all tests:

```bash
cargo test
```

Run unit and binary tests (cargo alias):

```bash
cargo test-unit
```

Run integration tests (cargo alias):

```bash
cargo test-integration
```
