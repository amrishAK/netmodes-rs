use super::socket_error::SocketError;

pub trait TcpSocketPlatform {
    type Socket: Copy;

    fn create_tcp_socket() -> Result<Self::Socket, SocketError>;
    fn configure_listener_socket(fd: Self::Socket) -> Result<(), SocketError>;
    fn bind_socket(fd: Self::Socket, host: &str, port: u16) -> Result<(), SocketError>;
    fn listen_socket(fd: Self::Socket, backlog: i32) -> Result<(), SocketError>;
    fn accept_connection(fd: Self::Socket) -> Result<Self::Socket, SocketError>;
    fn close_socket(fd: Self::Socket) -> Result<(), SocketError>;
    fn send_data(fd: Self::Socket, data: &[u8]) -> Result<usize, SocketError>;
    fn receive_data(fd: Self::Socket, buffer: &mut [u8]) -> Result<usize, SocketError>;
}