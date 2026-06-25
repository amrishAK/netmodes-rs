
use super::common::{
    as_socket, bind_ipv4_socket, close_ipv4_socket, ensure_winsock_initialized, wsa_last_error,
    Socket,
};
use crate::core::socket::socket_error::SocketError;
use crate::core::socket::tcp_platform::TcpSocketPlatform;
use std::mem;
use windows_sys::Win32::Networking::WinSock::{
    accept, listen, recv, send, setsockopt, socket, AF_INET, INVALID_SOCKET, IPPROTO_TCP,
    SOCK_STREAM, SOCKADDR, SOCKADDR_IN, SO_EXCLUSIVEADDRUSE, SOL_SOCKET, SOCKET_ERROR,
};

/// Windows implementation of the TCP socket platform abstraction.
pub struct WindowsTcpSocket;

impl TcpSocketPlatform for WindowsTcpSocket {
    type Socket = Socket;

    fn create_tcp_socket() -> Result<Self::Socket, SocketError> {
        ensure_winsock_initialized()?;

        let sock = unsafe { socket(AF_INET as i32, SOCK_STREAM, IPPROTO_TCP as i32) };

        if sock == INVALID_SOCKET {
            return Err(SocketError::CreateSocket(wsa_last_error()));
        }

        Ok(sock as Socket)
    }

    fn configure_listener_socket(fd: Self::Socket) -> Result<(), SocketError> {
        // Use exclusive address use on Windows instead of SO_REUSEADDR.
        // This prevents multiple listeners on the same port.
        let yes: i32 = 1;

        let ret = unsafe {
            setsockopt(
                as_socket(fd),
                SOL_SOCKET as i32,
                SO_EXCLUSIVEADDRUSE as i32,
                &yes as *const i32 as *const u8,
                mem::size_of::<i32>() as i32,
            )
        };

        if ret == SOCKET_ERROR {
            return Err(SocketError::SetSocketOption(wsa_last_error()));
        }

        Ok(())
    }

    fn bind_socket(fd: Self::Socket, host: &str, port: u16) -> Result<(), SocketError> {
        bind_ipv4_socket(fd, host, port)
    }

    fn listen_socket(fd: Self::Socket, backlog: i32) -> Result<(), SocketError> {
        let ret = unsafe { listen(as_socket(fd), backlog) };

        if ret == SOCKET_ERROR {
            return Err(SocketError::ListenSocket(wsa_last_error()));
        }

        Ok(())
    }

    fn accept_connection(fd: Self::Socket) -> Result<Self::Socket, SocketError> {
        let mut addr: SOCKADDR_IN = unsafe { mem::zeroed() };
        let mut addrlen: i32 = mem::size_of::<SOCKADDR_IN>() as i32;

        let client = unsafe {
            accept(
                as_socket(fd),
                &mut addr as *mut SOCKADDR_IN as *mut SOCKADDR,
                &mut addrlen,
            )
        };

        if client == INVALID_SOCKET {
            return Err(SocketError::AcceptConnection(wsa_last_error()));
        }

        Ok(client as Socket)
    }

    fn close_socket(fd: Self::Socket) -> Result<(), SocketError> {
        close_ipv4_socket(fd)
    }

    fn send_data(fd: Self::Socket, data: &[u8]) -> Result<usize, SocketError> {
        if data.len() > i32::MAX as usize {
            return Err(SocketError::SendData(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "buffer too large for send",
            )));
        }

        let ret = unsafe { send(as_socket(fd), data.as_ptr(), data.len() as i32, 0) };

        if ret == SOCKET_ERROR {
            return Err(SocketError::SendData(wsa_last_error()));
        }

        Ok(ret as usize)
    }

    fn receive_data(fd: Self::Socket, buffer: &mut [u8]) -> Result<usize, SocketError> {
        if buffer.len() > i32::MAX as usize {
            return Err(SocketError::ReceiveData(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "buffer too large for recv",
            )));
        }

        let ret = unsafe { recv(as_socket(fd), buffer.as_mut_ptr(), buffer.len() as i32, 0) };

        if ret == SOCKET_ERROR {
            return Err(SocketError::ReceiveData(wsa_last_error()));
        }

        Ok(ret as usize)
    }
}