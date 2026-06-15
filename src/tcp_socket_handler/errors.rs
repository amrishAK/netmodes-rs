use thiserror::Error;
use super::raw_socket::errors::SocketError;

#[derive(Error, Debug)]
pub enum TcpHandlerError {
    #[error("Tcp server creation failed due to invalid port: {0}")]
    TcpPortValidationError(String),

    #[error("Tcp server creation failed due to invalid host: {0}")]
    TcpHostValidationError(String),

    #[error("Tcp server creation failed due to invalid state: {0}")]
    TcpStateError(String),

    // Single #[from] for SocketError
    #[error("Tcp server socket error: {0}")]
    TcpSocketError(#[from] SocketError),

    #[error("Tcp client write operation failed due to socket error: {0}")]
    TcpClientWriteError(SocketError),
}