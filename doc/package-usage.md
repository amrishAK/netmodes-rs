# Package Usage

This guide covers practical usage patterns after you complete [getting-started.md](getting-started.md).

## Import Pattern

Most applications only need symbols from `tcp_socket_handler` and callback types from `core::models::domain::tcp`.

```rust
use std::sync::Arc;

use thugal_net::core::models::domain::tcp::{
    ContextHandler,
    OnMessageHandler,
    TcpClientSession,
};
use thugal_net::tcp_socket_handler::{
    Connected,
    Disconnected,
    TcpClientConfiguration,
    TcpClientHandler,
    TcpHandlerError,
    TcpServer,
    TcpServerConfiguration,
};
```

## Lifecycle Rules

Server lifecycle:

- `TcpServer::new(config)` -> `TcpServerHandler<Created>`
- `into_listening()` -> `TcpServerHandler<Listening>`
- `run(handler)` available in `Listening`

Client lifecycle:

- `TcpClientHandler::<Disconnected>::new(config)`
- `connect()` -> `TcpClientHandler<Connected>`
- `send()` and `receive()` available in `Connected`

These state transitions are compile-time enforced.

## Server Setup Pattern

Use a two-step startup: build with `TcpServer::new`, then transition to listening with `into_listening`.

```rust
let listening = TcpServer::new(TcpServerConfiguration {
    host: "127.0.0.1".to_string(),
    port: 8080,
    max_buffer_size: 1024,
})?
.into_listening()?;
```

At this point, only `run(handler)` is available. This prevents accidental usage in invalid lifecycle states.

## Handler-Centric Server Pattern

Use `OnMessageHandler` for all inbound payload handling. The callback receives:

- `TcpClientSession` for replying to the current client.
- `ContextHandler` for registry-level actions.
- Byte payload slice for message content.

```rust
use std::sync::Arc;
use thugal_net::core::models::domain::tcp::{
    ContextHandler,
    OnMessageHandler,
    TcpClientSession,
};

let handler: OnMessageHandler = Arc::new(
    |client: &TcpClientSession, context: ContextHandler, data: &[u8]| {
        let _ = client.reply(data);

        // Example: broadcast to all connected clients.
        let _ = context.broadcast_message(data);
    },
);
```

Typical handler behaviors:

- Echo: `client.reply(data)`
- Broadcast: `context.broadcast_message(data)`
- Targeted routing: parse payload, resolve `client_id`, call `send_message_to_client`

## Context Operations

`ContextHandler` exposes helper methods for connection-wide behavior:

- `send_message_to_client(client_id, message)`
- `broadcast_message(message)`

Use these to build custom routing logic without directly managing socket descriptors.

Example targeted message:

```rust
use uuid::Uuid;

fn send_private_message(context: ContextHandler, to_client: Uuid, payload: &[u8]) {
    let _ = context.send_message_to_client(to_client, payload);
}
```

## Client Pattern

```rust
fn request_response(payload: &[u8]) -> Result<Vec<u8>, TcpHandlerError> {
    let client = TcpClientHandler::<Disconnected>::new(TcpClientConfiguration {
        host: "127.0.0.1".to_string(),
        port: 8080,
        max_buffer_size: 1024,
    })?
    .connect()?;

    client.send(payload)?;
    client.receive()
}
```

If you need to keep a long-lived connection, retain the returned `TcpClientHandler<Connected>` and repeat `send`/`receive` per application protocol.

## End-to-End Minimal Flow

1. Start server and transition to `Listening`.
2. Register one `OnMessageHandler`.
3. Call `run(handler)` in server process.
4. Create client in `Disconnected` state.
5. `connect()` to get `Connected` state.
6. `send()` payload, then `receive()` response.
7. Handle `TcpHandlerError` branches for reconnect/fail-fast decisions.

## Configuration Guidance

- `host`: use a concrete bind/connect address (`127.0.0.1`, `0.0.0.0`, etc.).
- `port`: must be non-zero.
- `max_buffer_size`: controls per-read allocation and truncation behavior.

Pick `max_buffer_size` based on expected payload size. Large values increase per-connection memory use.

Recommended defaults for local development:

- `host = "127.0.0.1"`
- `port = 8080` (or another free port)
- `max_buffer_size = 1024` for small text/binary frames

For larger payloads, increase buffer size with awareness of memory cost per active connection.

## Error Handling in Usage Code

`TcpHandlerError` should drive policy:

- `InvalidHost` / `InvalidPort`: fail fast on startup.
- `ConnectionClosed`: reconnect client or drop session gracefully.
- `PartialWrite`: retry if your framing/protocol supports it.
- `Socket(...)`: log operation context and decide reconnect vs fail.

See [error-handling.md](error-handling.md) for variant-level guidance.

## Current Limitations

- Server `run` is an accept loop and does not return under normal operation.

For exact exported symbols, see [api-overview.md](api-overview.md).
