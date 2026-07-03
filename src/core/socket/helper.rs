
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

#[cfg(test)]
mod socket_helper_tests {
    use super::parse_ipv4_host;
    use crate::core::socket::SocketError;

    #[test]
    fn empty_host_returns_wildcard_success() {
        let parsed = parse_ipv4_host("").expect("empty host should parse as wildcard");
        assert_eq!(parsed, 0);
    }

    #[test]
    fn wildcard_host_returns_wildcard_success() {
        let parsed = parse_ipv4_host("0.0.0.0").expect("wildcard host should parse");
        assert_eq!(parsed, 0);
    }

    #[test]
    fn localhost_alias_parses_to_loopback_success() {
        let parsed = parse_ipv4_host("localhost").expect("localhost should parse");
        assert_eq!(parsed, u32::from_ne_bytes([127, 0, 0, 1]));
    }

    #[test]
    fn loopback_literal_parses_to_loopback_success() {
        let parsed = parse_ipv4_host("127.0.0.1").expect("loopback should parse");
        assert_eq!(parsed, u32::from_ne_bytes([127, 0, 0, 1]));
    }

    #[test]
    fn dotted_quad_literal_parses_success() {
        let parsed = parse_ipv4_host("192.168.1.10").expect("valid ipv4 should parse");
        assert_eq!(parsed, u32::from_ne_bytes([192, 168, 1, 10]));
    }

    #[test]
    fn invalid_host_returns_bind_socket_error_failure() {
        let err = parse_ipv4_host("not-an-ip").expect_err("invalid host should fail");

        match err {
            SocketError::BindSocket(io_err) => {
                assert_eq!(io_err.kind(), std::io::ErrorKind::InvalidInput);
            }
            other => panic!("expected BindSocket error, got {other:?}"),
        }
    }
}