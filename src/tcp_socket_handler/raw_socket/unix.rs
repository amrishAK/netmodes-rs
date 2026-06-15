use super::TcpSocketPlatform;
use super::errors::SocketError;
use libc::{accept, bind, listen, sockaddr, sockaddr_in, socket, socklen_t};
use std::mem;
use std::os::fd::RawFd;

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
    fn configure_no_sigpipe(fd: RawFd) -> std::io::Result<()> {
        let yes: i32 = 1;

        let ret = unsafe {
            libc::setsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_NOSIGPIPE,
                &yes as *const i32 as *const libc::c_void,
                mem::size_of::<i32>() as socklen_t,
            )
        };

        if ret < 0 {
            return Err(std::io::Error::last_os_error());
        }

        Ok(())
    }

    #[cfg(not(any(target_os = "macos", target_os = "ios")))]
    fn configure_no_sigpipe(_fd: RawFd) -> std::io::Result<()> {
        Ok(())
    }
}

impl TcpSocketPlatform for UnixTcpSocket {
    type Socket = RawFd;

    fn create_tcp_socket() -> Result<Self::Socket, SocketError> {
        let fd = unsafe { socket(libc::AF_INET, libc::SOCK_STREAM, 0) };

        if fd < 0 {
            return Err(SocketError::CreateSocketError(std::io::Error::last_os_error()));
        }

        Self::configure_no_sigpipe(fd).map_err(SocketError::CreateSocketError)?;

        Ok(fd)
    }

    fn configure_listener_socket(fd: Self::Socket) -> Result<(), SocketError> {
        let yes: i32 = 1;

        let ret = unsafe {
            libc::setsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_REUSEADDR,
                &yes as *const i32 as *const libc::c_void,
                mem::size_of::<i32>() as socklen_t,
            )
        };

        if ret < 0 {
            return Err(SocketError::SetSocketOptionError(
                std::io::Error::last_os_error(),
            ));
        }

        Ok(())
    }

    fn parse_ipv4_host(host: &str) -> Result<libc::in_addr, SocketError> 
    {
        let mut addr: libc::in_addr = unsafe { mem::zeroed() };

        // "0.0.0.0" or empty -> INADDR_ANY
        if host.is_empty() || host == "0.0.0.0" {
            addr.s_addr = (libc::INADDR_ANY as u32).to_be();
            return Ok(addr);
        }

        // Convert dotted-quad string to in_addr (e.g. "192.168.1.5")
        let cstr = std::ffi::CString::new(host).map_err(|_| {
            SocketError::BindSocketError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "host contains interior NUL",
            ))
        })?;

        let ret = unsafe {
            libc::inet_pton(
                libc::AF_INET,
                cstr.as_ptr(),
                &mut addr as *mut libc::in_addr as *mut libc::c_void,
            )
        };

        if ret != 1 {
            // 0 = invalid address string, -1 = errno
            let err = if ret == 0 {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid IPv4 address")
            } else {
                std::io::Error::last_os_error()
            };
            return Err(SocketError::BindSocketError(err));
        }

        Ok(addr)
    }

    fn bind_socket(fd: Self::Socket, host: &str, port: u16) -> Result<(), SocketError> 
    {
        let mut addr: sockaddr_in = unsafe { mem::zeroed() };
        addr.sin_family = libc::AF_INET as u16;
        addr.sin_port = port.to_be();
        addr.sin_addr = Self::parse_ipv4_host(host)?;

        let ret = unsafe {
            bind(
                fd,
                &addr as *const sockaddr_in as *const sockaddr,
                mem::size_of::<sockaddr_in>() as socklen_t,
            )
        };

        if ret < 0 {
            return Err(SocketError::BindSocketError(std::io::Error::last_os_error()));
        }

        Ok(())
    }

    fn listen_socket(fd: Self::Socket, backlog: i32) -> Result<(), SocketError> {
        let ret = unsafe { listen(fd, backlog) };

        if ret < 0 {
            return Err(SocketError::ListenSocketError(std::io::Error::last_os_error()));
        }

        Ok(())
    }

    fn accept_connection(fd: Self::Socket) -> Result<Self::Socket, SocketError> {
        let mut addr: sockaddr_in = unsafe { mem::zeroed() };
        let mut addrlen: socklen_t = mem::size_of::<sockaddr_in>() as socklen_t;

        let client_fd = unsafe {
            accept(
                fd,
                &mut addr as *mut sockaddr_in as *mut sockaddr,
                &mut addrlen,
            )
        };

        if client_fd < 0 {
            return Err(SocketError::AcceptConnectionError(std::io::Error::last_os_error()));
        }

        Self::configure_no_sigpipe(client_fd).map_err(SocketError::AcceptConnectionError)?;

        Ok(client_fd)
    }

    fn close_socket(fd: Self::Socket) -> Result<(), SocketError> {
        let ret = unsafe { libc::close(fd) };

        if ret < 0 {
            return Err(SocketError::CloseSocketError(std::io::Error::last_os_error()));
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
            return Err(SocketError::SendDataError(std::io::Error::last_os_error()));
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
            return Err(SocketError::ReceiveDataError(std::io::Error::last_os_error()));
        }

        Ok(ret as usize)
    }
}