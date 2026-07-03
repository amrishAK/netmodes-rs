/// Error types produced by TCP server/client helpers.
pub mod errors;
/// Server-side lifecycle and runtime context modules.
pub mod server;
/// Client-side per-connection handler module.
pub mod client;

/// Unified error returned by TCP handler operations.
pub use errors::TcpHandlerError;
/// Public TCP settings and runtime types.
pub use server::{
	Created,
	Listening,
	TcpPeerConnection,
	TcpServer,
	TcpServerHandler,
	TcpServerConfiguration,
	Unconfigured,
};
pub use client::{
    Connected,
    Disconnected,
    TcpClientConfiguration,
    TcpClientHandler,
};