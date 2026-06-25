use super::super::helper;
use super::super::socket_error::SocketError;
use std::mem;
use std::os::windows::io::RawSocket;
use std::sync::OnceLock;
use windows_sys::Win32::Networking::WinSock::{
    bind, closesocket, WSAGetLastError, WSAStartup, AF_INET, SOCKADDR, SOCKADDR_IN, SOCKET,
    SOCKET_ERROR, WSADATA,
};

pub type Socket = RawSocket;

pub(super) fn as_socket(raw: Socket) -> SOCKET {
    raw as SOCKET
}

pub(super) fn wsa_last_error() -> std::io::Error {
    let code = unsafe { WSAGetLastError() };
    std::io::Error::from_raw_os_error(code)
}

/// Initialize WinSock once per process before creating sockets.
pub(super) fn ensure_winsock_initialized() -> Result<(), SocketError> {
    static WINSOCK_INIT: OnceLock<i32> = OnceLock::new();

    let ret = *WINSOCK_INIT.get_or_init(|| unsafe {
        let mut wsa_data: WSADATA = mem::zeroed();
        WSAStartup(0x0202, &mut wsa_data)
    });

    if ret == 0 {
        Ok(())
    } else {
        Err(SocketError::CreateSocket(std::io::Error::from_raw_os_error(ret)))
    }
}

pub(super) fn bind_ipv4_socket(fd: Socket, host: &str, port: u16) -> Result<(), SocketError> {
    let in_addr = helper::parse_ipv4_host(host)?;

    let mut addr: SOCKADDR_IN = unsafe { mem::zeroed() };
    addr.sin_family = AF_INET as u16;
    addr.sin_port = port.to_be();
    addr.sin_addr.S_un.S_addr = in_addr;

    let ret = unsafe {
        bind(
            as_socket(fd),
            &addr as *const SOCKADDR_IN as *const SOCKADDR,
            mem::size_of::<SOCKADDR_IN>() as i32,
        )
    };

    if ret == SOCKET_ERROR {
        return Err(SocketError::BindSocket(wsa_last_error()));
    }

    Ok(())
}

pub(super) fn close_ipv4_socket(fd: Socket) -> Result<(), SocketError> {
    let ret = unsafe { closesocket(as_socket(fd)) };

    if ret == SOCKET_ERROR {
        return Err(SocketError::CloseSocket(wsa_last_error()));
    }

    Ok(())
}