use uuid::Uuid;
use crate::core::socket::*;
use super::errors::TcpHandlerError;
use std::thread::spawn;


/// Lifecycle states for a TCP server instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpServerState {
    /// Server instance was created but not yet initialized.
    Created,
    /// Socket is bound to host and port.
    Bound,
    /// Socket is actively listening for new connections.
    Listening,
    /// Socket has been closed and cannot be reused.
    Closed,
}

/// Host/port configuration used to create a TCP server.
pub struct TcpSettings {
    /// IPv4 host or hostname understood by the socket layer.
    pub host: String,
    /// Listening port.
    pub port: u16,
}

/// Connected TCP client handle.
pub struct TcpClient {
    /// Stable identifier for logging and connection tracking.
    pub id: Uuid,
    fd: Socket,
}

/// TCP server wrapper managing socket lifecycle and accept loop.
pub struct TcpServer {
    /// Immutable server bind/listen settings.
    pub settings: TcpSettings,
    /// Stable identifier for server instance tracking.
    pub id: Uuid,
    fd: Socket,
    /// Current server lifecycle state.
    pub state: TcpServerState,
}

impl Drop for TcpServer{
    fn drop(&mut self) {
        if self.state != TcpServerState::Closed {
            _ = close_socket(self.fd);
            self.state = TcpServerState::Closed;
        }
    }
}

impl Drop for TcpClient{
    fn drop(&mut self) {
        _ = close_socket(self.fd);
    }
}

impl TcpClient {
    /// Read bytes from the client socket into the provided buffer.
    ///
    /// Returns `ConnectionClosed` when the peer performs an orderly shutdown.
    pub fn read(&self, buffer: &mut [u8]) -> Result<usize, TcpHandlerError> {
        let bytes_received = receive_data(self.fd, buffer)?;
        if bytes_received == 0 {
            return Err(TcpHandlerError::ConnectionClosed);
        }
        Ok(bytes_received)
    }

    /// Write the full buffer to the socket, failing on partial sends.
    pub fn write(&self, data: &[u8]) -> Result<(), TcpHandlerError> 
    {
        let bytes_sent = send_data(self.fd, data)?; // SocketError -> TcpHandlerError via From

        if bytes_sent != data.len() {
            return Err(TcpHandlerError::PartialWrite {
                sent: bytes_sent,
                total: data.len(),
            });
        }
        
        Ok(())
    }

    /// Attempt a single socket send and return the number of bytes sent.
    pub fn write_partial(&self, data: &[u8]) -> Result<usize, TcpHandlerError> 
    {
        let bytes_sent = send_data(self.fd, data)?; // SocketError -> TcpHandlerError via From
        Ok(bytes_sent)
    }
}

impl TcpServer {

    /// Create a server with validated settings and an open TCP socket.
    pub fn new(settings: TcpSettings) -> Result<Self, TcpHandlerError> {
        
        // Validate port and host
        if settings.port == 0 {
            return Err(TcpHandlerError::InvalidPort(format!("Invalid port number: {}", settings.port)));
        }

        if settings.host.is_empty() {
            return Err(TcpHandlerError::InvalidHost(format!("Invalid host: {}", settings.host)));
        }
        
        // Create the TCP socket
        let fd = create_tcp_socket()?;
        
        // Return the TcpServer instance
        let server = TcpServer {
            settings,
            id: Uuid::new_v4(),
            fd,
            state: TcpServerState::Created,
        };

        Ok(server)
    }

    /// Configure, bind, and place the server socket into listening mode.
    pub fn initialize(&mut self) -> Result<(), TcpHandlerError> {
        
        if self.state != TcpServerState::Created
        {
            return Err(TcpHandlerError::InvalidState(format!("Invalid state for initialization: {:?}", self.state)));
        }

        // Configure listener socket options before bind/listen.
        configure_listener_socket(self.fd)?;

        // Bind the socket to the specified host and port
        bind_socket(self.fd, &self.settings.host, self.settings.port)?;
        self.state = TcpServerState::Bound;

        // Start listening for incoming connections
        listen_socket(self.fd, 128)?;
        self.state = TcpServerState::Listening;
        Ok(())
    }
    
    
    /// Accept clients in a loop and invoke the handler on a dedicated thread.
    pub fn run<H>(&self, handler: H) -> Result<(), TcpHandlerError> where H: Fn(TcpClient) + Send + Copy + 'static,
    {
        if self.state != TcpServerState::Listening {
            return Err(TcpHandlerError::InvalidState(format!("Invalid state for running server: {:?}", self.state)));
        }

        loop {
            let client_fd = accept_connection(self.fd)?; // blocking

            let client = TcpClient {
                id: Uuid::new_v4(),
                fd: client_fd,
            };

            // user-provided connection handler
            spawn(move || {
                handler(client);
            });
        }
    }
}

#[cfg(test)]
mod tcp_handlers_tests {
    use super::{TcpServer, TcpServerState, TcpSettings};
    use crate::tcp_socket_handler::TcpHandlerError;
    use uuid::Uuid;

    #[test]
    fn new_with_zero_port_returns_invalid_port_failure() {
        let settings = TcpSettings {
            host: "127.0.0.1".to_string(),
            port: 0,
        };

        let err = match TcpServer::new(settings) {
            Ok(_) => panic!("port 0 should be rejected"),
            Err(err) => err,
        };

        match err {
            TcpHandlerError::InvalidPort(msg) => {
                assert!(msg.contains("Invalid port number"));
            }
            other => panic!("expected InvalidPort, got {other:?}"),
        }
    }

    #[test]
    fn new_with_empty_host_returns_invalid_host_failure() {
        let settings = TcpSettings {
            host: "".to_string(),
            port: 8080,
        };

        let err = match TcpServer::new(settings) {
            Ok(_) => panic!("empty host should be rejected"),
            Err(err) => err,
        };

        match err {
            TcpHandlerError::InvalidHost(msg) => {
                assert!(msg.contains("Invalid host"));
            }
            other => panic!("expected InvalidHost, got {other:?}"),
        }
    }

    #[test]
    fn initialize_when_closed_state_returns_invalid_state_failure() {
        let mut server = TcpServer {
            settings: TcpSettings {
                host: "127.0.0.1".to_string(),
                port: 8080,
            },
            id: Uuid::new_v4(),
            fd: 0,
            state: TcpServerState::Closed,
        };

        let err = server
            .initialize()
            .expect_err("initialize should fail outside Created state");

        match err {
            TcpHandlerError::InvalidState(msg) => {
                assert!(msg.contains("Invalid state for initialization"));
            }
            other => panic!("expected InvalidState, got {other:?}"),
        }

        assert_eq!(server.state, TcpServerState::Closed);
    }

    #[test]
    fn run_when_closed_state_returns_invalid_state_failure() {
        let server = TcpServer {
            settings: TcpSettings {
                host: "127.0.0.1".to_string(),
                port: 8080,
            },
            id: Uuid::new_v4(),
            fd: 0,
            state: TcpServerState::Closed,
        };

        let err = server
            .run(|_| {})
            .expect_err("run should fail outside Listening state");

        match err {
            TcpHandlerError::InvalidState(msg) => {
                assert!(msg.contains("Invalid state for running server"));
            }
            other => panic!("expected InvalidState, got {other:?}"),
        }
    }
}

