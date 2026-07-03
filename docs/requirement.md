# thugal-net Requirements

This document defines the end-goal target architecture and capabilities for this project. Items here are target outcomes and may not all be implemented in the current release.

## 1. Product Direction

### 1.1 Purpose

thugal-net is a lightweight, event-driven L4 (transport) server framework for Rust, designed for native nonblocking I/O and pluggable L7 (application) protocol support.

Primary focus:

- Provide a stable, ergonomic L4 foundation using `epoll` on Linux and equivalent platform abstractions on Windows.
- Expose low-level control where needed while keeping high-level usage simple.
- Support pluggable L7 protocol handlers via trait-based codec system.
- Remain lightweight and sync-first: no embedded async runtime, no fixed dependency on Tokio.
- Support cross-platform behavior through well-defined abstractions.

### 1.2 Product Goals

- Build a package-first API for TCP and UDP workflows.
- Keep a strict layered architecture:
  - Layer 1: low-level platform-specific socket and polling primitives.
  - Layer 2: mid-level transport handlers and lifecycle orchestration.
  - Layer 3: high-level user API and callbacks.
- Support custom binary protocol integration via codec interfaces.
- Keep pub-sub out of the L4 core and provide it as a separate L7 package/plugin.
- Support session management as a first-class capability.
- Provide lifecycle hooks for connection events:
  - on_connect
  - on_disconnect
  - on_message

### 1.3 Non-Goals

- This project is not a load balancer.
- This project is not a full message broker with persistence and durability.
- This project is not tied to learning outcomes as primary scope.

---

## 2. Architecture Requirements

### 2.1 Layer 1: Low-Level Platform-Specific Core

Responsibilities:

- Native socket creation, bind/listen/accept/connect, send/recv, close.
- Platform-specific behavior isolated behind a common trait/API.
- Polling/event primitives for readiness notification.

Target requirements:

- Poller abstraction with platform backends:
  - Linux: epoll
  - macOS: kqueue
  - Windows: WSAPoll
- Non-blocking socket configuration as default for poll-driven flows.
- Readiness registration API (register, modify, unregister, wait).
- Cross-platform TCP socket wrapper for Unix and Windows.
- Unified TCP socket abstraction exposed through core socket modules.
- Centralized error mapping through SocketError.

### 2.2 Layer 2: Mid-Level Handler Layer

Responsibilities:

- Server/client lifecycle orchestration.
- Connection session management.
- Event dispatch from low-level events into user callbacks.
- Protocol decode/encode pipeline integration.
- Client and session state coordination.

Target requirements:

- Dedicated on_connect and on_disconnect callbacks in the server API.
- Handler execution model integrated with poller-based event loop.
- Mid-level protocol adapter wiring for custom protocol codecs.
- First-class session registry wiring in server and context flows.
- TCP server typestate flow:
  - Created -> Listening
- TCP client typestate flow:
  - Disconnected -> Connected
- Session loop with message callback invocation.
- Shared context handler and client registry.

### 2.3 Layer 3: High-Level User API

Responsibilities:

- Ergonomic package API for application developers.
- Clear configuration structs/builders.
- Stable callback contracts and error behavior.

Target requirements:

- First-class event hook registration for on_connect and on_disconnect.
- Protocol selection/codec configuration in public server and client configuration.
- End-user examples for default protocol and custom protocol integration.
- Public API for user-defined session models and session lifecycle management.
- Public TCP server and TCP client APIs.
- Stable message callback contract consumed by server flows.

---

## 3. Functional Requirements

### 3.1 Core Scope Targets

- TCP server lifecycle and run loop.
- TCP client connect/send/receive flow.
- Registry-backed connection context and broadcast helper.
- Cross-platform TCP socket operations for Unix and Windows.
- UDP handler public API as a stable core surface (server and client handlers).
- Pub-sub is intentionally outside core scope and will be delivered as an L7 package.

### 3.2 Expanded Capability Targets

- TCP as a stable core API.
- Poller-backed event loop foundations with epoll/kqueue/WSAPoll.
- Custom protocol support:
  - Protocol frame abstraction.
  - Codec trait(s) for decode/encode.
  - Default built-in codec for basic framing.
- Lifecycle hooks:
  - on_connect(client, context) when a client session is accepted and registered.
  - on_disconnect(client, context, reason) when a client session closes or is removed.
  - on_message(client, context, payload) for inbound payload handling.
- Session management support:
  - Session registry for user-defined session types.
  - Session create/update/remove flows attached to connection lifecycle.
  - Session lookup APIs for handler and application usage.

### 3.3 Deferred Scope

- Broadcast and multicast behavior.
- Cluster mode and peer replication.
- MQTT extension.
- Built-in pub-sub implementation in this crate (moved to separate L7 package/plugin).

Deferred items are not removed from long-term roadmap, but they are not blocking for the package-focused core release.

---

## 4. Technical Requirements

### 4.1 Dependency and Runtime Constraints

- Use standard Rust plus low-level socket crates as needed.
- Do not require Tokio runtime for core network I/O.
- Keep platform-specific code isolated behind core abstraction boundaries.

### 4.2 Polling Abstraction

The codebase must introduce an internal poller module with a unified API, for example:

- Poller::new()
- Poller::register(fd, interest)
- Poller::modify(fd, interest)
- Poller::unregister(fd)
- Poller::wait(timeout)

Platform mapping:

- Linux: epoll backend.
- macOS: kqueue backend.
- Windows: WSAPoll backend.

### 4.3 Protocol Abstraction

The package must support pluggable protocol implementations:

- Frame-level data model for decoded payloads and metadata.
- Codec interface for decode and encode.
- Clear error propagation for malformed frames.
- Ability to plug a user-defined codec per handler/server configuration.
- Pub-sub protocol primitives are out of core scope and belong to separate L7 packages/plugins.

### 4.4 Lifecycle Hook Contract

Server-side handler contract must include:

- on_connect(client, context)
- on_message(client, context, payload)
- on_disconnect(client, context, reason)

Hooks must be safe to call from concurrent session handling paths and must not leak low-level platform details.

Context contract:

- Context exposes the client registry and the session registry.
- For TCP, session registry support is first-class.
- For UDP, session registry support is optional and protocol-dependent.

### 4.5 Session Registry Contract

The package must expose a session registry that allows user-defined session models.

Required behavior:

- Users define session structs and their session IDs.
- Registry supports add, get, contains, remove, list IDs, snapshot, clear.
- Registry uses thread-safe access suitable for concurrent handlers.
- Session registry API naming is session-oriented and separate from client registry.

Integration requirements:

- Session lifecycle can be hooked from on_connect/on_disconnect.
- Context layer can access both client and session registries.
- Session IDs and client IDs can be mapped by user logic when needed.

---

## 5. Client Registry, Session Store, and Context

### 5.1 Client Registry

- Internal map of active client connections: `client_id -> client_metadata`.
- Metadata: remote address, protocol handler, etc.
- Minimal, read-only after connection acceptance.
- Used internally by the framework for connection tracking.

### 5.2 Session Store

- Per-connection application state (user-defined).
- Optional: some L7 protocols need it; others don't.
- Attached to connection context.
- Users define the session struct; registry provides get/set/remove operations.

### 5.3 Relationship: Client Registry vs. Session Store

- **Client registry:** Framework-owned, immutable metadata. Managed by the L4 core.
- **Session store:** User-owned, mutable application state. Managed by L7 protocol handlers or user callbacks.
- **Both accessible in callbacks via connection context.**

## 6. API and Interface Requirements

### 6.1 Public Package API

- Preserve typestate safety for server/client lifecycle transitions.
- Expose clear configuration objects for host/port/buffer/protocol/hook registration.
- Keep breaking changes explicit and versioned.

### 6.2 Examples and Documentation

- Provide minimal TCP echo example.
- Provide lifecycle hooks example.
- Provide custom codec example.
- Provide session registry example with a user-defined session struct.
- Document thread-safety and callback execution behavior.

---

## 7. Callbacks and Synchronous Execution Model

### 7.1 Callback Lifecycle Hooks

- `on_connect(client, context)`
- `on_message(client, context, message_bytes)` (TCP)
- `on_datagram(client, context, message_bytes)` (UDP)
- `on_disconnect(client, context, reason)`

### 7.2 Synchronous Callback Guarantee

- TCP/UDP event processing is poller-driven (epoll/kqueue/WSAPoll).
- Callbacks execute synchronously from the poll/event loop path.
- Framework-managed thread-per-peer execution is out of scope for target architecture.

Callback constraints:

- Callbacks are synchronous and must remain fast.
- They must not use `async`/`await`.
- They must not block indefinitely or perform blocking I/O.
- They must not hold locks longer than necessary.

**Lock Pattern:**
1. Lock registry.
2. Read/clone needed data.
3. Unlock registry immediately.
4. Run callback with cloned data.

**For users needing async work:**
- Spawn a background task/thread and use `std::sync::mpsc::channel` to communicate results.
- See Phase 4 (async wrapper crate) for ergonomic async support via higher-level libraries.

---

## 8. Success Criteria

The package is considered successful for the end-goal release when:

1. Product clarity
   - Requirements and docs position thugal-net as a reusable package, not a learning exercise.

2. Three-layer architecture enforcement
   - Clear separation exists between platform core, handler layer, and user API layer.

3. Poller foundation
   - Unified polling API is available with epoll implementation complete and wired.
   - kqueue and WSAPoll adapters are defined with equivalent behavior contracts.

4. Lifecycle hooks
   - on_connect, on_message, on_disconnect are available and documented.

5. Session management
  - Session registry is available for user-defined session structs.
  - Session create/read/update/remove flows are documented and tested.

6. Custom protocol support
   - At least one built-in codec exists and user-defined codec integration is possible.

7. UDP support
  - UDP server and client handler APIs are available and documented.

8. Package quality
   - Unit and integration tests cover lifecycle transitions, callback execution, and protocol encode/decode.
   - Documentation includes migration and usage guidance for consumers.
  - Unit tests cover session registry behavior for user-defined sessions.

---

## 9. L4 Transport Differences (TCP vs UDP)

### 9.1 TCP

- Connection-oriented; `on_connect` and `on_disconnect` callbacks.
- Session registry supported and recommended for stateful L7 protocols.
- Ordered, reliable, stream-based delivery.
- Client registry maintains active TCP connection state.

### 9.2 UDP

- Datagram-oriented; `on_datagram` callback (no `on_connect`/`on_disconnect` by default).
- Session registry is **configurable:** users can enable it for custom session tracking if their L7 protocol requires it.
- Unordered, unreliable delivery; L7 protocols add reliability/ordering if needed (e.g., QUIC, CoAP).
- Each datagram is processed independently unless L7 protocol implements session semantics.

---

## 10. Open Questions

- Default framing format for the built-in codec.
- Backpressure and partial-write policy in poll-driven mode.
- Canonical mapping strategy between client identity and session identity.

---

## 11. Evolution Path & Async Support

### 11.1 Phase 1: Core L4 Server (This Package)

- Sockets, epoll/Windows event loop, reactor.
- Client registry + session store.
- Callback-based lifecycle hooks (sync).
- TCP and UDP support.

### 11.2 Phase 2: L7 Protocol Plugins

- Trait-based codec system for L7 protocols.
- Example L7 protocols: minimal HTTP, pub/sub, custom protocols.

### 11.3 Phase 3: Documentation & Examples

- Comprehensive examples for TCP echo, callbacks, codec integration.
- Session registry examples with user-defined session types.

### 11.4 Phase 4: Async Wrapper (Separate Crates, Optional)

**Important:** Async support is **not part of this package.** Instead, separate wrapper crates provide ergonomic async/await APIs:

- **`thugal-net-tokio`** (or similar): Wraps `thugal-net` for Tokio runtime users.
  - Provides `async fn accept()`, `recv()`, `send()`.
  - Spawns reactor in background thread, bridges callbacks to async tasks via channels.

- **`thugal-net-async-std`** (or similar): Wraps `thugal-net` for async-std users.
  - Same pattern: async wrapper over sync reactor.

**Design Principle:**
- This package remains 100% sync, event-driven, no embedded runtime.
- Users choose their async runtime (Tokio, async-std, or none) via wrapper crates.
- Async is a consumer concern, not a core concern.

---

## 12. Revision History

- v3.0 - L4/L7 event-driven framework reposition.
  - Repositioned project as lightweight L4 foundation with pluggable L7 protocols.
  - Clarified sync-first design: no async in core, optional async via wrapper crates.
  - Demoted pub-sub from framework feature to reference L7 protocol example.
  - Made UDP session support configurable (protocol-dependent).
  - Clarified callback model: synchronous, reactor-thread execution.
  - Separated client registry (framework-owned) from session store (user-owned).
  - Defined L4 transport differences (TCP vs. UDP).
  - Moved async to Phase 4 as separate optional wrapper crates (thugal-net-tokio, thugal-net-async-std).

- v2.0 - Package-focused rewrite.
  - Repositioned project as reusable package for external users.
  - Introduced explicit three-layer architecture requirements.
  - Promoted UDP support to required package scope.
  - Promoted pub-sub protocol support to required package scope.
  - Added poller and epoll/kqueue/WSAPoll requirement track.
  - Added lifecycle hooks requirements (on_connect/on_disconnect/on_message).
  - Added session management requirements and session registry contract.
  - Added custom protocol/codec support requirements.

- v1.0 - Initial learning-focused requirements.

- v3.1 - Core/L7 boundary and callback model alignment.
  - Defined pub-sub as an L7 package/plugin, not a core L4 requirement.
  - Standardized hooks to use `(client, context)` with context-level registry access.
  - Defined poller-driven callback execution and removal of framework-managed thread-per-peer execution in target architecture.
  - Removed legacy gap-analysis section and older gap references.
