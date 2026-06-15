use thiserror::Error;

#[derive(Error, Debug)]
pub enum SocketError {
    // Single #[from] for std::io::Error
    #[error("I/O error")]
    Io(#[from] std::io::Error),

    #[error("Failed to create socket")]
    CreateSocketError(std::io::Error),

    #[error("Failed to bind socket")]
    BindSocketError(std::io::Error),

    #[error("Failed to listen on socket")]
    ListenSocketError(std::io::Error),

    #[error("Failed to accept connection")]
    AcceptConnectionError(std::io::Error),

    #[error("Failed to close socket")]
    CloseSocketError(std::io::Error),

    #[error("Failed to send data")]
    SendDataError(std::io::Error),

    #[error("Failed to receive data")]
    ReceiveDataError(std::io::Error),

    #[error("Failed to set socket option")]
    SetSocketOptionError(std::io::Error),
}