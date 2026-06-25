/// Error type for socket operations across platform backends.
pub mod socket_error;
mod helper;
mod tcp_platform;

/// Unified socket error exposed to callers.
pub use socket_error::SocketError;
/// Trait describing platform-specific TCP socket behavior.
pub use tcp_platform::TcpSocketPlatform;

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
pub(crate) type Platform = unix::tcp::UnixTcpSocket;
#[cfg(windows)]
pub(crate) type Platform = windows::tcp::WindowsTcpSocket;

pub(crate) type Socket = <Platform as TcpSocketPlatform>::Socket;

pub(crate) fn create_tcp_socket() -> Result<Socket, SocketError> {
    Platform::create_tcp_socket()
}

pub(crate) fn configure_listener_socket(fd: Socket) -> Result<(), SocketError> {
    Platform::configure_listener_socket(fd)
}

pub(crate) fn bind_socket(fd: Socket, host: &str, port: u16) -> Result<(), SocketError> {
    Platform::bind_socket(fd, host, port)
}

pub(crate) fn listen_socket(fd: Socket, backlog: i32) -> Result<(), SocketError> {
    Platform::listen_socket(fd, backlog)
}

pub(crate) fn accept_connection(fd: Socket) -> Result<Socket, SocketError> {
    Platform::accept_connection(fd)
}

pub(crate) fn close_socket(fd: Socket) -> Result<(), SocketError> {
    Platform::close_socket(fd)
}

pub(crate) fn send_data(fd: Socket, data: &[u8]) -> Result<usize, SocketError> {
    Platform::send_data(fd, data)
}

pub(crate) fn receive_data(fd: Socket, buffer: &mut [u8]) -> Result<usize, SocketError> {
    Platform::receive_data(fd, buffer)
}