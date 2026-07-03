use super::common::{bind_ipv4_socket, Socket};
use crate::core::socket::socket_error::SocketError;
use crate::core::socket::tcp_platform::TcpSocketPlatform;
use std::{mem, os::fd::RawFd};
use libc::{accept, listen, AF_INET, SOCK_STREAM};

/// Unix implementation of the TCP socket platform abstraction.
pub struct UnixTcpSocket;

impl UnixTcpSocket {
    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "dragonfly"
    ))]
    fn send_flags() -> i32 {
        libc::MSG_NOSIGNAL
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "android",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "dragonfly"
    )))]
    fn send_flags() -> i32 {
        0
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    fn configure_no_sigpipe(fd: RawFd) -> Result<(), SocketError> {
        let yes: i32 = 1;

        let ret = unsafe {
            libc::setsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_NOSIGPIPE,
                &yes as *const i32 as *const libc::c_void,
                mem::size_of::<i32>() as libc::socklen_t,
            )
        };

        if ret < 0 {
            return Err(SocketError::ConfigureNoSigpipe(std::io::Error::last_os_error()));
        }

        Ok(())
    }

    #[cfg(not(any(target_os = "macos", target_os = "ios")))]
    fn configure_no_sigpipe(_fd: RawFd) -> Result<(), SocketError> {
        Ok(())
    }
}

impl TcpSocketPlatform for UnixTcpSocket {
    type Socket = Socket;

    fn create_tcp_socket() -> Result<Self::Socket, SocketError> {
        let fd = unsafe { libc::socket(AF_INET, SOCK_STREAM, 0) };
        if fd < 0 {
            return Err(SocketError::CreateSocket(std::io::Error::last_os_error()));
        }

        Self::configure_no_sigpipe(fd)?;

        Ok(fd)
    }

    fn configure_listener_socket(fd: Self::Socket) -> Result<(), SocketError> {
        // Set SO_REUSEADDR to allow rebinding to the same address/port
        let yes: i32 = 1;
        let ret = unsafe {
            libc::setsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_REUSEADDR,
                &yes as *const i32 as *const libc::c_void,
                mem::size_of::<i32>() as libc::socklen_t,
            )
        };

        if ret < 0 {
            return Err(SocketError::SetSocketOption(std::io::Error::last_os_error()));
        }

        Ok(())
    }

    fn bind_socket(fd: Self::Socket, host: &str, port: u16) -> Result<(), SocketError> {
        bind_ipv4_socket(fd, host, port)
    }

    fn listen_socket(fd: Self::Socket, backlog: i32) -> Result<(), SocketError> {
        let ret = unsafe { listen(fd, backlog) };
        if ret < 0 {
            return Err(SocketError::ListenSocket(std::io::Error::last_os_error()));
        }
        Ok(())
    }

    fn accept_connection(fd: Self::Socket) -> Result<Self::Socket, SocketError> {
        loop {
            let client_fd = unsafe { accept(fd, std::ptr::null_mut(), std::ptr::null_mut()) };
            if client_fd >= 0 {
                return Ok(client_fd);
            }

            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }

            return Err(SocketError::AcceptConnection(err));
        }
    }

    fn connect_socket(fd: Self::Socket, host: &str, port: u16) -> Result<(), SocketError> {
        let mut addr: libc::sockaddr_in = unsafe { std::mem::zeroed() };
        addr.sin_family = libc::AF_INET as u16;
        addr.sin_port = port.to_be();
        addr.sin_addr = super::common::get_ipv4_host(host)?;

        loop {
            let ret = unsafe {
                libc::connect(
                    fd,
                    &addr as *const libc::sockaddr_in as *const libc::sockaddr,
                    std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t,
                )
            };

            if ret == 0 {
                return Ok(());
            }

            let err = std::io::Error::last_os_error();
            
            if err.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }

            return Err(SocketError::ConnectSocket(err));
        }
    }

    fn close_socket(fd: Self::Socket) -> Result<(), SocketError> {
        let ret = unsafe { libc::close(fd) };
        if ret < 0 {
            return Err(SocketError::CloseSocket(std::io::Error::last_os_error()));
        }
        Ok(())
    }

    fn send_data(fd: Self::Socket, data: &[u8]) -> Result<usize, SocketError> {
        let ret = unsafe {
            libc::send(
                fd,
                data.as_ptr() as *const libc::c_void,
                data.len(),
                Self::send_flags(),
            )
        };

        if ret < 0 {
            return Err(SocketError::SendData(std::io::Error::last_os_error()));
        }

        Ok(ret as usize)
    }

    fn receive_data(fd: Self::Socket, buffer: &mut [u8]) -> Result<usize, SocketError> {
        let ret = unsafe {
            libc::recv(
                fd,
                buffer.as_mut_ptr() as *mut libc::c_void,
                buffer.len(),
                0,
            )
        };

        if ret < 0 {
            return Err(SocketError::ReceiveData(std::io::Error::last_os_error()));
        }

        Ok(ret as usize)
    }
}