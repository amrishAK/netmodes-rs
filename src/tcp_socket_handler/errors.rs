use thiserror::Error;
use crate::core::socket::SocketError;

/// Errors returned by TCP server lifecycle and client I/O operations.
#[derive(Error, Debug)]
pub enum TcpHandlerError {
    #[error("TCP client not found: {0}")]
    ClientNotFound(String),

    #[error("TCP client lock poisoned")]
    ClientLockError,

    #[error("TCP context lock poisoned")]
    ContextLockError,

    #[error("TCP server creation failed: invalid port: {0}")]
    InvalidPort(String),

    #[error("TCP server creation failed: invalid host: {0}")]
    InvalidHost(String),

    #[error("TCP server creation failed: invalid state: {0}")]
    InvalidState(String),

    #[error("TCP client registration failed: {0}")]
    RegistrationError(String),

    #[error("TCP socket error: {0}")]
    Socket(#[from] SocketError),

    #[error("Partial write: sent {sent} of {total} bytes")]
    PartialWrite { sent: usize, total: usize },

    #[error("Connection closed by peer")]
    ConnectionClosed,
}