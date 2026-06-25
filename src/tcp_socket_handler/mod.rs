/// Error types produced by TCP server/client helpers.
pub mod errors;
/// TCP server and client abstractions built on the core socket layer.
pub mod tcp_handler;

/// Unified error returned by TCP handler operations.
pub use errors::TcpHandlerError;
/// Public TCP settings and runtime types.
pub use tcp_handler::{TcpSettings, TcpServer, TcpClient};