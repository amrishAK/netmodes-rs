/// Error types produced by TCP server/client helpers.
pub mod errors;
/// TCP server and client abstractions built on the core socket layer.
pub mod server_handler;

pub(crate) mod client_handler;
pub(crate) mod context_handler;
pub(crate) use client_handler::tcp_client_handler;

/// Unified error returned by TCP handler operations.
pub use errors::TcpHandlerError;
/// Public TCP settings and runtime types.
pub use server_handler::{Created, Listening, TcpClient, TcpServer, TcpServerHandler, TcpServerConfiguration, TcpSettings, Unconfigured};