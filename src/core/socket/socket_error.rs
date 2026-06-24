use thiserror::Error;

#[derive(Debug, Error)]
pub enum SocketError {
	#[error("failed to create socket: {0}")]
	CreateSocket(std::io::Error),

	#[error("failed to set socket option: {0}")]
	SetSocketOption(std::io::Error),

	#[error("failed to configure no-sigpipe: {0}")]
	ConfigureNoSigpipe(std::io::Error),

	#[error("failed to bind socket: {0}")]
	BindSocket(std::io::Error),

	#[error("failed to listen on socket: {0}")]
	ListenSocket(std::io::Error),

	#[error("failed to accept connection: {0}")]
	AcceptConnection(std::io::Error),

	#[error("failed to close socket: {0}")]
	CloseSocket(std::io::Error),

	#[error("failed to send data: {0}")]
	SendData(std::io::Error),

	#[error("failed to receive data: {0}")]
	ReceiveData(std::io::Error),
}
