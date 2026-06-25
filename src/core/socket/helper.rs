
use super::socket_error::SocketError;
use std::net::Ipv4Addr;

/// Parse supported IPv4 host strings into a 32-bit address representation.
///
/// Accepts wildcard and localhost aliases in addition to dotted-quad literals.
pub(crate) fn parse_ipv4_host(host: &str) -> Result<u32, SocketError> {
    if host.is_empty() || host == "0.0.0.0" {
        return Ok(0);
    }

    if host == "localhost" || host == "127.0.0.1" {
        return Ok(u32::from_ne_bytes([127, 0, 0, 1]));
    }

    let ip = host.parse::<Ipv4Addr>().map_err(|err| {
        SocketError::BindSocket(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            err,
        ))
    })?;

    // Keep bytes laid out as network-order octets in memory.
    Ok(u32::from_ne_bytes(ip.octets()))
}