use super::super::helper;
use super::super::socket_error::SocketError;
use libc::{in_addr, sockaddr, sockaddr_in, socklen_t, AF_INET};
use std::{mem, os::fd::RawFd};

pub type Socket = RawFd;


/// Build an `in_addr` value with network-byte-order contents for libc calls.
pub(super) fn get_ipv4_host(host: &str) -> Result<in_addr, SocketError> {
    let host_addr = helper::parse_ipv4_host(host)?;

    Ok(in_addr {
        // Keep the same in-memory network-order octets produced by parse_ipv4_host.
        s_addr: host_addr,
    })
}

pub(super) fn bind_ipv4_socket(fd: Socket, host: &str, port: u16) -> Result<(), SocketError> {
    
    let mut addr: sockaddr_in = unsafe { mem::zeroed() };
    addr.sin_family = AF_INET as u16;
    addr.sin_port = port.to_be();
    addr.sin_addr = get_ipv4_host(host)?;

    let ret = unsafe {
        libc::bind(fd, &addr as *const sockaddr_in as *const sockaddr,  mem::size_of::<sockaddr_in>() as socklen_t)
    };

    if ret < 0 {
        return Err(SocketError::BindSocket(std::io::Error::last_os_error()));
    }

    Ok(())
}

pub(super) fn close_ipv4_socket(fd: Socket) -> Result<(), SocketError> {
    let ret = unsafe { libc::close(fd) };
    
    if ret < 0 {
        return Err(SocketError::CloseSocket(std::io::Error::last_os_error()));
    }
    
    Ok(())
}