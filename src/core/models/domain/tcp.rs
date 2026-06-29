use std::marker::PhantomData;
use std::sync::{Arc, RwLock};

use uuid::Uuid;

use crate::core::registry::client_registry::ClientRegistry;
use crate::core::socket::Socket;

/// Configuration for a TCP server instance.
///
/// Specifies the bind address, port, and I/O buffer size.
pub struct TcpServerConfiguration {
    pub host: String,
    pub port: u16,
    /// Maximum buffer size for client I/O operations.
    pub max_buffer_size: usize,
}

/// A TCP server instance with a unique ID and socket file descriptor.
pub struct TcpServer {
    pub config: TcpServerConfiguration,
    pub id: Uuid,
    pub(crate) fd: Socket,
}

/// A TCP client connection with a unique ID and socket file descriptor.
pub struct TcpClient {
    pub id: Uuid,
    pub(crate) fd: Socket,
}

/// Runtime context holding the server state and client registry.
pub struct TcpContext {
    pub server_id: Uuid,
    pub(crate) registry: ClientRegistry<Uuid, TcpClient>,
    /// Maximum buffer size for client I/O operations.
    pub max_buffer_size: usize,
}

/// Type-state wrapper for a TCP server with compile-time state tracking.
///
/// States: `Unconfigured` → `Created` → `Listening`
/// 
/// This ensures only valid operations are available at each stage:
/// - `Created`: call `into_listening()` to transition
/// - `Listening`: call `run()` to start accepting connections
pub struct TcpServerHandler<State> {
    pub server: TcpServer,
    pub(crate) context: Arc<TcpContext>,
    pub(crate) _state: PhantomData<State>,
}

/// Handle to the TCP runtime context for managing all clients.
///
/// Supports registry lookups, message broadcasting, and centralized client state.
#[derive(Clone)]
pub struct ContextHandler(pub(crate) Arc<TcpContext>);

/// Handle to a single TCP client connection.
///
/// Provides methods to send data back to the client and retrieve its ID.
#[derive(Clone)]
pub struct ClientHandler(pub(crate) Arc<RwLock<TcpClient>>);

/// Handler invoked when client data arrives.
///
/// Receives:
/// - `&ClientHandler`: Interface to send replies to this client
/// - `ContextHandler`: Access to all clients for broadcast or registry lookups
/// - `&[u8]`: Received data
pub type OnMessageHandler = Arc<dyn Fn(&ClientHandler, ContextHandler, &[u8]) + Send + Sync + 'static>;