# Getting Started

This guide gets you to a first successful TCP request/response with `thugal-net`.

## Prerequisites

- Rust toolchain installed.
- A free local TCP port (examples use `8080`).
- Two terminals (one for server, one for client).

## 1. Add the dependency

If you are consuming from crates.io:

```toml
[dependencies]
thugal-net = "0.1.0"
```

If you are developing against a local checkout:

```toml
[dependencies]
thugal-net = { path = "../thugal-net" }
```

## 2. Start a minimal echo server

```rust
use std::sync::Arc;

use thugal_net::core::models::domain::tcp::{
    ContextHandler,
    OnMessageHandler,
    TcpClientSession,
};
use thugal_net::tcp_socket_handler::{
    TcpServer,
    TcpServerConfiguration,
};

fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let listening = TcpServer::new(TcpServerConfiguration {
        host: "127.0.0.1".to_string(),
        port: 8080,
        max_buffer_size: 1024,
    })?
    .into_listening()?;

    let handler: OnMessageHandler = Arc::new(
        |client: &TcpClientSession, _context: ContextHandler, data: &[u8]| {
            let _ = client.reply(data);
        },
    );

    listening.run(handler)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_server()
}
```

Run the server in terminal 1:

```bash
cargo run --bin server
```

## 3. Connect and send from a client

```rust
use thugal_net::tcp_socket_handler::{
    Disconnected,
    TcpClientConfiguration,
    TcpClientHandler,
};

fn run_client() -> Result<(), Box<dyn std::error::Error>> {
    let client = TcpClientHandler::<Disconnected>::new(TcpClientConfiguration {
        host: "127.0.0.1".to_string(),
        port: 8080,
        max_buffer_size: 1024,
    })?
    .connect()?;

    client.send(b"hello")?;
    let response = client.receive()?;

    println!("{}", String::from_utf8_lossy(&response));
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_client()
}
```

In terminal 2, run your client binary and send `hello`.

Expected output:

```text
hello
```

## 4. Verify in this repository

If you are running from this repository:

1. Start sample server binary:

```bash
cargo run --bin server
```

2. In another terminal, run all tests:

```bash
cargo test
```

3. Optionally run only TCP integration tests:

```bash
cargo test --test tcp_client_integration
cargo test --test tcp_server_integration
cargo test --test tcp_client_server_high_integration
```

## Troubleshooting

- `Address already in use`: change the port in server and client configurations.
- Connection refused: ensure the server process is running before starting client calls.
- Garbled output: print with `String::from_utf8_lossy` (as shown) when testing text payloads.

For deeper usage patterns, see [package-usage.md](package-usage.md).
