# netmodes-rs Requirements

## 1. Purpose and Goals

### 1.1 Purpose

`netmodes-rs` is a learning project to understand Rust networking, socket APIs, and async runtime internals. The project implements:

- A single server node that supports multiple clients.
- Multiple network message modes:
  - TCP and UDP
  - Unicast, broadcast, multicast
  - Pub-sub (publish/subscribe)
- Optional cluster mode for experimentation.
- A thin CLI client to send messages in different modes.

The project is **not** a full-scale load balancer. That is a separate project (Thendra).

### 1.2 Primary Goals

- Learn and practice:
  - Low-level socket programming in Rust (`std::net`, `socket2`).
  - Non-blocking I/O and event loops.
  - Building a custom async runtime that supports `async`/`await`.
  - TCP and UDP communication patterns.
  - Broadcast and multicast socket configuration.
  - Basic pub-sub message routing.
- Build a small, ergonomic network library and CLI for experimentation.

### 1.3 Non-Goals

- Not a production-grade load balancer.
- Not a general-purpose async runtime (like Tokio).
- Not a full message broker with durability, queues, or complex protocols.
- Not focused on security, authentication, or encryption (can be added later as exercises).

---

## 2. Functional Requirements

### 2.1 Network Modes

The system must support the following network modes:

| Mode        | Protocol | Description                                         |
|-------------|----------|-----------------------------------------------------|
| echo        | TCP/UDP  | Server echoes back received messages to the client. |
| unicast     | UDP      | Server sends messages to a single client address.   |
| broadcast   | UDP      | Server sends messages to all clients on the LAN.    |
| multicast   | UDP      | Server sends messages to a multicast group.         |
| pub         | TCP/UDP  | Client publishes a message to a topic.              |
| sub         | TCP/UDP  | Client subscribes to a topic and receives messages. |

The server must:

- Accept connections/messages from multiple clients concurrently.
- Route messages based on mode and topic (for pub-sub).
- Support both TCP and UDP for echo and pub-sub.
- Support UDP for unicast, broadcast, and multicast.

### 2.2 Server (node)

The `node` binary must:

- Listen on configurable TCP and UDP ports.
- Maintain in-memory subscription state for pub-sub:
  - `topic -> Vec<client>` mappings.
- Implement message routing:
  - For echo: send back the same message.
  - For unicast: send to a specific client address.
  - For broadcast: send to a broadcast address.
  - For multicast: send to a multicast group address.
  - For pub-sub: forward messages to all subscribers of a topic.
- Support cluster mode:
  - Multiple server instances with redundancy.
  - Connect to peer nodes.
  - Forward messages to peers with duplicate prevention (message ID or TTL).
  - Maintain client states across replicas so that if one server goes down, clients can reconnect to another replica.

### 2.3 Thin CLI Client (thin-cli)

The `thin-cli` binary must:

- Provide subcommands or flags for modes:
  - `echo`, `unicast`, `broadcast`, `multicast`, `pub`, `sub`.
- Support protocol selection:
  - `--proto tcp` or `--proto udp`.
- Support target configuration:
  - `--addr` for server address.
  - `--topic` for pub-sub topics.
  - `--bcast` for broadcast address.
  - `--maddr` for multicast address.
- Allow users to:
  - Send messages from stdin or command-line arguments.
  - Receive and print messages from the server.

Example usage:

```bash
thin-cli echo --proto udp --addr 127.0.0.1:9000
thin-cli unicast --proto udp --addr 192.168.1.10:9000
thin-cli broadcast --proto udp --bcast 192.168.1.255:9000
thin-cli multicast --proto udp --maddr 239.0.0.1:9000
thin-cli pub --proto tcp --topic chat --addr 127.0.0.1:9100
thin-cli sub --proto tcp --topic chat --addr 127.0.0.1:9100
```

### 2.4 Message Format

The system must use a **binary message format** with a basic protocol:

```
<cmd> <option> <message>
```

Where:

- `cmd`: 1-byte command code (e.g., `0x01` = SUB, `0x02` = PUB, `0x03` = ECHO, etc.).
- `option`: 1–4 bytes of optional flags or metadata (e.g., topic length, QoS, TTL).
- `message`: variable-length payload (e.g., topic name, message content).

The exact binary protocol details (cmd codes, option layout, length encoding) will be **designed later** during implementation. This section defines the high-level structure only.

The format must be:

- Compact and efficient for network transport.
- Easy to parse with minimal overhead.
- Extensible for future features (new cmds, options).

### 2.5 Pub-Sub Protocol

#### Core: Queue-Based Pub-Sub

The primary pub-sub protocol must be queue-based:

- Simple in-memory routing: `topic -> Vec<Sender<Message>>`.
- Binary protocol:
  - `cmd = 0x01` – SUBSCRIBE `<topic>`
  - `cmd = 0x02` – UNSUBSCRIBE `<topic>`
  - `cmd = 0x03` – PUBLISH `<topic> <payload>`
- Server forwards messages immediately to all subscribers of a topic.
- No persistence, no QoS, no wildcards.

#### Extension: MQTT Pub-Sub (Optional)

The server may optionally support MQTT-based pub-sub as an extension:

- MQTT runs on a **separate TCP port** (e.g., `9183`).
- Queue-based pub-sub is the primary protocol.
- MQTT is optional, not required for core learning goals.
- Queue-based and MQTT pub-sub maintain **separate topic state** (isolated):
  - Subscriptions are not shared between queue-based and MQTT.
  - Topic routing is independent per protocol.
- MQTT support includes:
  - Basic CONNECT, SUBSCRIBE, UNSUBSCRIBE, PUBLISH, DISCONNECT.
  - No QoS 2, no complex session state, no retain flags initially.

Protocols are clearly separated by port:

- Default port (e.g., `9100`) → queue-based pub-sub (binary protocol).
- MQTT port (e.g., `9183`) → MQTT-based pub-sub (binary protocol).

---

## 3. Technical Requirements

### 3.1 Core Constraint: Pure Sockets + Custom Async Runtime

The project must:

- Use only:
  - `std::net` sockets and/or `socket2` for low-level socket operations.
  - A custom, manually implemented async runtime.
- Not use:
  - Tokio's runtime, or any equivalent high-level async runtime (e.g., `async-std`, `smol` runtime) for network I/O.
- Support:
  - `async`/`await` syntax via the custom runtime as a wrapper around futures.
- Implement:
  - Non-blocking sockets.
  - A **single event loop** that handles both TCP and UDP.
  - A polling mechanism:
    - `epoll` on Linux.
    - `kqueue` on macOS.
    - `WSAPoll` on Windows.
    - (Optional) `io_uring` on Linux as an advanced option.
  - A task/spawner model and waker mechanism to drive futures.
  - Async helpers for:
    - TCP accept/read/write.
    - UDP recv/send (including broadcast/multicast).
    - Optional timers for timeouts.

The runtime must be **minimal and specific to netmodes-rs**, not a general-purpose async library.

### 3.2 Platform-Specific Polling Code

Platform-specific polling code must be handled using a **wrapper crate implemented from scratch**:

- Create your own small wrapper crate that:
  - Exposes a unified polling API (e.g., `Poller::new()`, `Poller::wait()`, `Poller::register()`).
  - Internally uses:
    - `epoll` on Linux.
    - `kqueue` on macOS.
    - `WSAPoll` on Windows.
- Use conditional compilation (`#[cfg(target_os = "...")]`) inside the wrapper crate only.
- The rest of the codebase uses the unified API without platform-specific code.
- Do not use an existing third-party polling crate; implement it yourself for learning.

This keeps the main codebase clean and portable while isolating platform differences, and ensures you understand how polling mechanisms work.

### 3.3 Cluster Mode: Redundancy

Cluster mode must provide **multiple servers with basic redundancy and client state maintenance**:

- Multiple `node` instances can run on different hosts/ports.
- Nodes:
  - Connect to peer nodes.
  - Forward messages to peers with duplicate prevention (message ID or TTL).
  - Maintain client states across replicas:
    - If one server goes down, clients can reconnect to another replica.
    - Replicas share or replicate client subscription state.
- Redundancy is **basic**:
  - No complex consensus or leader election.
  - No persistent storage or durable queues.
  - Simple peer list configuration (e.g., `peers = [...]`).
  - Peer heartbeats and failure detection will be detailed later during implementation.
  - Initial implementation can use simple timeouts and periodic heartbeats.

This is "enough for fun" and learning, not production-grade clustering.

### 3.4 Allowed Dependencies

Allowed crates:

- `socket2` – for low-level socket configuration.
- `futures` – for `Future` trait, `FutureExt`, and basic utilities (optional).
- `clap` – for CLI parsing.
- `log` + `env_logger` – for logging.
- Standard library (`std`) only for non-async core logic.

Not allowed for core I/O:

- Tokio runtime (`tokio::main`, `tokio::spawn`, etc.).
- `async-std` runtime.
- `smol` runtime.
- High-level HTTP frameworks as the primary network abstraction.
- Third-party polling crates (you must implement the polling wrapper crate yourself).

### 3.5 Platform Support

The system must:

- Run on Linux, macOS, and Windows.
- Use appropriate polling mechanisms per platform via your own wrapper crate:
  - Linux: `epoll`.
  - macOS: `kqueue`.
  - Windows: `WSAPoll`.

Broadcast and multicast support must:

- Use standard socket options (`set_broadcast`, `join_multicast_v4/v6`).
- Be documented with notes on LAN requirements and OS differences.

### 3.6 Async Runtime Requirements

The custom async runtime must:

- Provide:
  - `Runtime::spawn(future)` to schedule a future.
  - `Runtime::run()` to drive the event loop.
- Implement:
  - A task queue or ready list.
  - A waker mechanism to mark tasks as ready when socket events occur.
  - Mapping between socket events and tasks.
  - A **single event loop** for both TCP and UDP.
- Expose async-like APIs:
  - `TcpListener::accept().await`
  - `TcpStream::read().await`, `TcpStream::write().await`
  - `UdpSocket::recv().await`, `UdpSocket::send().await`

These APIs must be implemented against the custom runtime, not Tokio.

---

## 4. Interface Requirements

### 4.1 CLI Interface

The `thin-cli` must provide:

- A subcommand-based interface:
  - `thin-cli echo ...`
  - `thin-cli unicast ...`
  - `thin-cli broadcast ...`
  - `thin-cli multicast ...`
  - `thin-cli pub ...`
  - `thin-cli sub ...`
- Common flags:
  - `--proto tcp|udp`
  - `--addr <host:port>`
  - `--topic <name>`
  - `--bcast <host:port>`
  - `--maddr <host:port>`
  - `--help`

### 4.2 Server Configuration

The `node` must support:

- A configuration file or CLI args for:
  - TCP port (queue-based pub-sub).
  - UDP port.
  - MQTT TCP port (optional).
  - Broadcast address (optional).
  - Multicast group address (optional).
  - Peer addresses for cluster mode.

Example config (TOML or JSON):

```toml
tcp_port = 9100
udp_port = 9000
mqtt_port = 9183
broadcast_addr = "192.168.1.255:9000"
multicast_addr = "239.0.0.1:9000"
peers = ["192.168.1.10:9100", "192.168.1.11:9100"]
```

---

## 5. Success Criteria

The project is considered complete for learning when:

1. **Core networking**
   - The node:
     - Accepts TCP and UDP connections.
     - Implements echo, unicast, broadcast, multicast.
     - Implements queue-based pub-sub with topic-based routing.
   - The thin-cli:
     - Can send and receive messages in all modes.
     - Supports both TCP and UDP for echo and pub-sub.
   - Binary message format with `<cmd> <option> <message>` is implemented.

2. **Custom async runtime**
   - The async runtime:
     - Uses non-blocking sockets.
     - Implements a **single event loop** with `epoll`/`kqueue`/`WSAPoll` via your own wrapper crate.
     - Spawns and drives futures without Tokio runtime.
     - Provides async `accept`, `read`, `write`, `recv`, `send` APIs.

3. **Cluster mode (optional)**
   - If implemented:
     - Multiple `node` instances run with basic redundancy.
     - Nodes forward messages to peers with duplicate prevention.
     - Client states are maintained across replicas.
     - Clients can reconnect to another replica if one server goes down.
     - No complex consensus or leader election.
     - Peer heartbeats and failure detection are simple (to be detailed later).

4. **MQTT extension (optional)**
   - If implemented:
     - MQTT runs on a separate port.
     - Basic CONNECT, SUBSCRIBE, UNSUBSCRIBE, PUBLISH work.
     - Queue-based and MQTT pub-sub maintain **separate topic state** (isolated).

5. **Learning outcomes**
   - You understand:
     - How sockets and non-blocking I/O work.
     - How event loops and polling mechanisms operate.
     - How async runtimes manage tasks and wake futures.
     - How TCP and UDP differ in behavior and API usage.
     - How broadcast and multicast are configured and used.
     - How pub-sub routing is implemented at a basic level.
     - How basic cluster redundancy with client state maintenance works.
     - How polling mechanisms (epoll/kqueue/WSAPoll) work by implementing your own wrapper.

6. **Code quality**
   - Code is:
     - Modular and documented.
     - Easy to extend (e.g., add new modes, MQTT, or cluster features).
     - Testable with unit tests for core logic.
   - Platform-specific code is isolated in your own wrapper crate.

---

## 6. Open Questions

These can be decided during implementation:

- Exact binary protocol details (cmd codes, option layout, length encoding) – to be designed later.
- Peer heartbeat interval and failure detection timeout – to be detailed later.
- How client states are replicated across peers (simple vs more robust).
- How deep MQTT support should go (basic vs more features).

---

## 7. Revision History

- v1.0 – Initial requirements for `netmodes-rs`.
  - Goals: learning networking and async runtime internals.
  - Core constraint: pure sockets + custom async runtime, no Tokio runtime for I/O.
  - Modes: echo, unicast, broadcast, multicast, pub, sub.
  - Optional cluster mode with redundancy and client state maintenance.
  - Queue-based pub-sub as primary protocol.
  - MQTT as optional extension on a separate port (isolated topic state).
  - Binary message format: `<cmd> <option> <message>` (details designed later).
  - Single runtime for TCP and UDP.
  - Platform-specific polling via your own wrapper crate (from scratch).
  - Cluster mode: multiple servers with basic redundancy, client states replicated, heartbeats detailed later.
