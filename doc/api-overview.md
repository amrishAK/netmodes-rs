# API Overview

This document summarizes the current public API surface of `netmodes-rs`.

## Crate Exports

Top-level crate modules:

- `netmodes_rs::core`
- `netmodes_rs::tcp_socket_handler`

Current status:

- TCP server/client API is implemented and intended for use.

## Primary User-Facing TCP API

Main import path:

```rust
use netmodes_rs::tcp_socket_handler::*;
```

Key exported types:

- Server lifecycle:
  - `TcpServer`
  - `TcpServerConfiguration`
  - `TcpServerHandler<State>`
  - `Created`, `Listening`, `Unconfigured`
  - `TcpPeerConnection`
- Client lifecycle:
  - `TcpClientHandler<State>`
  - `TcpClientConfiguration`
  - `Disconnected`, `Connected`
- Errors:
  - `TcpHandlerError`

### Server Typestate Flow

```rust
TcpServer::new(config)
	-> Result<TcpServerHandler<Created>, TcpHandlerError>

TcpServerHandler<Created>::into_listening()
	-> Result<TcpServerHandler<Listening>, TcpHandlerError>

TcpServerHandler<Listening>::run(handler)
	-> Result<(), TcpHandlerError>
```

Additional server handler methods:

- `TcpServerHandler<State>::config() -> &TcpServerConfiguration`
- `TcpServerHandler<State>::id() -> uuid::Uuid`

### Client Typestate Flow

```rust
TcpClientHandler::<Disconnected>::new(config)
	-> Result<TcpClientHandler<Disconnected>, TcpHandlerError>

TcpClientHandler<Disconnected>::connect()
	-> Result<TcpClientHandler<Connected>, TcpHandlerError>

TcpClientHandler<Connected>::send(data)
	-> Result<(), TcpHandlerError>

TcpClientHandler<Connected>::receive()
	-> Result<Vec<u8>, TcpHandlerError>
```

Additional client handler methods:

- `TcpClientHandler<State>::id() -> uuid::Uuid`
- `TcpClientHandler<State>::config() -> &TcpClientConfiguration`

## Callback and Context Types

Callback-related domain types are defined in `netmodes_rs::core::models::domain::tcp`.

- `OnMessageHandler`
- `TcpClientSession`
- `ContextHandler`

`OnMessageHandler` signature:

```rust
Arc<dyn Fn(&TcpClientSession, ContextHandler, &[u8]) + Send + Sync + 'static>
```

Callback intent:

- `TcpClientSession` represents the current connected peer session.
- `ContextHandler` provides shared server context access (including registry-backed client context).

## Registry Surface

Registry modules are available under `netmodes_rs::core::registry`.

- `client_registry::ClientRegistry<Key, T>`
- `register_error::RegistryError`

## Error Types

Top-level TCP errors:

- `InvalidPort`, `InvalidHost`, `InvalidState`
- `ClientNotFound`, `ClientLockError`, `ContextLockError`
- `RegistrationError`
- `PartialWrite`, `ConnectionClosed`
- `Socket(SocketError)`

Low-level socket errors are represented by `netmodes_rs::core::socket::SocketError`.

For handling patterns and examples, see [error-handling.md](error-handling.md).
