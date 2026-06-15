use super::TcpSocketPlatform;
use super::errors::SocketError;
use std::mem;
use std::net::Ipv4Addr;
use std::os::windows::io::RawSocket;
use std::sync::OnceLock;
use windows_sys::Win32::Networking::WinSock::{
    accept, bind, closesocket, listen, recv, send, setsockopt, socket, WSAStartup, WSADATA,
    AF_INET, INADDR_ANY, INVALID_SOCKET, IPPROTO_TCP, SOCK_STREAM, SOCKADDR, SOCKADDR_IN,
    SO_EXCLUSIVEADDRUSE, SOL_SOCKET, SOCKET, SOCKET_ERROR,
};

pub struct WindowsTcpSocket;

impl WindowsTcpSocket {
    #[inline]
    fn as_socket(raw: RawSocket) -> SOCKET {
        raw as SOCKET
    }

    fn ensure_winsock_initialized() -> Result<(), SocketError> {
        static WINSOCK_INIT: OnceLock<i32> = OnceLock::new();

        let ret = *WINSOCK_INIT.get_or_init(|| unsafe {
            let mut wsa_data: WSADATA = mem::zeroed();
            WSAStartup(0x0202, &mut wsa_data)
        });

        if ret == 0 {
            Ok(())
        } else {
            Err(SocketError::CreateSocketError(std::io::Error::from_raw_os_error(ret)))
        }
    }

    fn parse_ipv4_host_windows(host: &str) -> Result<u32, SocketError>
    {
        // "" or 0.0.0.0 => INADDR_ANY
        if host.is_empty() || host == "0.0.0.0" {
            return Ok(INADDR_ANY);
        }

        let ip = host.parse::<Ipv4Addr>().map_err(|err| {
            SocketError::BindSocketError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                err,
            ))
        })?;

        // Keep bytes laid out as network-order octets in memory.
        Ok(u32::from_ne_bytes(ip.octets()))
    }
}

impl TcpSocketPlatform for WindowsTcpSocket {
    type Socket = RawSocket;

    fn create_tcp_socket() -> Result<Self::Socket, SocketError> {
        Self::ensure_winsock_initialized()?;

        let sock = unsafe {
            socket(
                AF_INET as i32,
                SOCK_STREAM,
                IPPROTO_TCP as i32,
            )
        };

        if sock == INVALID_SOCKET {
            return Err(SocketError::CreateSocketError(
                std::io::Error::last_os_error(),
            ));
        }

        Ok(sock as RawSocket)
    }

    fn configure_listener_socket(fd: Self::Socket) -> Result<(), SocketError> {
        // Use exclusive address use on Windows instead of SO_REUSEADDR.
        // This prevents multiple listeners on the same port.
        let yes: i32 = 1;

        let ret = unsafe {
            setsockopt(
                Self::as_socket(fd),
                SOL_SOCKET as i32,
                SO_EXCLUSIVEADDRUSE as i32,
                &yes as *const i32 as *const u8,
                mem::size_of::<i32>() as i32,
            )
        };

        if ret == SOCKET_ERROR {
            return Err(SocketError::SetSocketOptionError(
                std::io::Error::last_os_error(),
            ));
        }

        Ok(())
    }
    fn bind_socket(fd: Self::Socket, host: &str, port: u16) -> Result<(), SocketError> 
    {
        let in_addr = Self::parse_ipv4_host_windows(host)?;

        let mut addr: SOCKADDR_IN = unsafe { mem::zeroed() };
        addr.sin_family = AF_INET as u16;
        addr.sin_port = port.to_be();
        addr.sin_addr.S_un.S_addr = in_addr;

        let ret = unsafe {
            bind(
                Self::as_socket(fd),
                &addr as *const SOCKADDR_IN as *const SOCKADDR,
                mem::size_of::<SOCKADDR_IN>() as i32,
            )
        };

        if ret == SOCKET_ERROR {
            return Err(SocketError::BindSocketError(
                std::io::Error::last_os_error(),
            ));
        }

        Ok(())
    }

    fn listen_socket(fd: Self::Socket, backlog: i32) -> Result<(), SocketError> {
        let ret = unsafe { listen(Self::as_socket(fd), backlog) };

        if ret == SOCKET_ERROR {
            return Err(SocketError::ListenSocketError(
                std::io::Error::last_os_error(),
            ));
        }

        Ok(())
    }

    fn accept_connection(fd: Self::Socket) -> Result<Self::Socket, SocketError> {
        let mut addr: SOCKADDR_IN = unsafe { mem::zeroed() };
        let mut addrlen: i32 = mem::size_of::<SOCKADDR_IN>() as i32;

        let client = unsafe {
            accept(
                Self::as_socket(fd),
                &mut addr as *mut SOCKADDR_IN as *mut SOCKADDR,
                &mut addrlen,
            )
        };

        if client == INVALID_SOCKET {
            return Err(SocketError::AcceptConnectionError(
                std::io::Error::last_os_error(),
            ));
        }

        Ok(client as RawSocket)
    }

    fn close_socket(fd: Self::Socket) -> Result<(), SocketError> {
        let ret = unsafe { closesocket(Self::as_socket(fd)) };

        if ret == SOCKET_ERROR {
            return Err(SocketError::CloseSocketError(
                std::io::Error::last_os_error(),
            ));
        }

        Ok(())
    }

    fn send_data(fd: Self::Socket, data: &[u8]) -> Result<usize, SocketError> {
        if data.len() > i32::MAX as usize {
            return Err(SocketError::SendDataError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "buffer too large for send",
            )));
        }

        let ret = unsafe {
            send(
                Self::as_socket(fd),
                data.as_ptr(),
                data.len() as i32,
                0,
            )
        };

        if ret == SOCKET_ERROR {
            return Err(SocketError::SendDataError(
                std::io::Error::last_os_error(),
            ));
        }

        Ok(ret as usize)
    }

    fn receive_data(fd: Self::Socket, buffer: &mut [u8]) -> Result<usize, SocketError> {
        if buffer.len() > i32::MAX as usize {
            return Err(SocketError::ReceiveDataError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "buffer too large for recv",
            )));
        }

        let ret = unsafe {
            recv(
                Self::as_socket(fd),
                buffer.as_mut_ptr(),
                buffer.len() as i32,
                0,
            )
        };

        if ret == SOCKET_ERROR {
            return Err(SocketError::ReceiveDataError(
                std::io::Error::last_os_error(),
            ));
        }

        Ok(ret as usize)
    }
}