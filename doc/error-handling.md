# Error Handling

`netmodes-rs` exposes two primary error layers so you can decide whether to recover, retry, or fail fast.

## Error Layers

### `TcpHandlerError`

Application-facing error type used by TCP client/server handlers.

Common variants:

- `InvalidPort`
- `InvalidHost`
- `InvalidState`
- `RegistrationError`
- `ClientNotFound`
- `ClientLockError`
- `ContextLockError`
- `PartialWrite`
- `ConnectionClosed`
- `Socket(...)` (wraps `SocketError`)

### `SocketError`

Low-level socket operation error from the core socket layer.

Common variants:

- `CreateSocket`
- `BindSocket`
- `ListenSocket`
- `AcceptConnection`
- `ConnectSocket`
- `SendData`
- `ReceiveData`
- `CloseSocket`

## How to Decide Quickly

- Validation/startup errors (`InvalidHost`, `InvalidPort`, bind/listen/create failures): fail fast.
- Runtime transport churn (`ConnectionClosed`, transient accept/connect/read/write failures): reconnect or continue based on role.
- State/registry/lock issues (`InvalidState`, `RegistrationError`, `ClientLockError`, `ContextLockError`): log with identifiers and surface to operators.
- `PartialWrite`: retry write completion if protocol allows; otherwise close and re-establish.

## Practical Pattern

```rust
match client.send(payload) {
    Ok(()) => {}
    Err(TcpHandlerError::ConnectionClosed) => {
        // Expected in long-running systems; trigger reconnect policy.
        reconnect_client();
    }
    Err(TcpHandlerError::PartialWrite { .. }) => {
        // Retry or reconnect depending on your framing guarantees.
        retry_send(payload);
    }
    Err(TcpHandlerError::Socket(socket_err)) => {
        eprintln!("send failed at socket layer: {socket_err}");
        apply_transport_policy(socket_err);
    }
    Err(err) => {
        // Non-recoverable or policy-specific branch.
        eprintln!("send failed: {err}");
    }
}
```

## Recommended Recovery Policy

- Startup path: validate host/port early and abort on invalid configuration.
- Accept loop/server run path: continue on transient `AcceptConnection` failures with bounded backoff.
- Client path: on `ConnectSocket`/`ConnectionClosed`, reconnect with jittered backoff.
- Write path: on `PartialWrite`, either finish the remaining bytes or reconnect before retransmission.
- Read path: treat EOF/closed connection as a normal disconnect event.

## Logging Checklist

Always include:

- role (`server` or `client`)
- operation (`connect`, `accept`, `send`, `receive`, `close`)
- peer identity (`client_id`, remote endpoint, or session id)
- error variant and inner socket error (if wrapped)
- retry decision (`retry`, `reconnect`, `fail_fast`, `drop_session`)

## Best Practices

- Keep retry logic centralized so behavior is consistent.
- Use bounded exponential backoff with jitter for reconnects.
- Treat disconnects as expected events in production.
- Record enough metadata to diagnose lock and registry contention.
