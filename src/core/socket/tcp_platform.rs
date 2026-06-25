use super::socket_error::SocketError;

/// Platform abstraction for creating and using IPv4 TCP sockets.
pub trait TcpSocketPlatform {
    /// Native socket handle type used by the current platform.
    type Socket: Copy;

    /// Create a stream-oriented IPv4 socket.
    fn create_tcp_socket() -> Result<Self::Socket, SocketError>;
    /// Apply listener-specific socket options before bind/listen.
    fn configure_listener_socket(fd: Self::Socket) -> Result<(), SocketError>;
    /// Bind a socket to the provided host and port.
    fn bind_socket(fd: Self::Socket, host: &str, port: u16) -> Result<(), SocketError>;
    /// Mark a bound socket as listening with the provided backlog.
    fn listen_socket(fd: Self::Socket, backlog: i32) -> Result<(), SocketError>;
    /// Accept one incoming client connection.
    fn accept_connection(fd: Self::Socket) -> Result<Self::Socket, SocketError>;
    /// Close a socket handle.
    fn close_socket(fd: Self::Socket) -> Result<(), SocketError>;
    /// Send bytes on a connected socket.
    fn send_data(fd: Self::Socket, data: &[u8]) -> Result<usize, SocketError>;
    /// Read bytes from a connected socket into the provided buffer.
    fn receive_data(fd: Self::Socket, buffer: &mut [u8]) -> Result<usize, SocketError>;
}