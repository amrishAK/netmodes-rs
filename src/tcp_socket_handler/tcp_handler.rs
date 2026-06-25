use std::marker::PhantomData;
use std::thread::spawn;
use uuid::Uuid;

pub use crate::core::models::domain::state_type::{Created, Listening, Unconfigured};
use crate::core::socket::*;
pub use crate::core::models::domain::tcp::{TcpClient, TcpServer, TcpServerHandler, TcpServerConfiguration};
pub type TcpSettings = TcpServerConfiguration;

use super::errors::TcpHandlerError;

impl Drop for TcpClient{
    fn drop(&mut self) {
        _ = close_socket(self.fd);
    }
}

impl Drop for TcpServer {
    fn drop(&mut self) {
        _ = close_socket(self.fd);
    }
}

impl<State> TcpServerHandler<State> {
    pub fn config(&self) -> &TcpServerConfiguration {
        &self.server.config
    }

    pub fn id(&self) -> Uuid {
        self.server.id
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
    /// Create a new TCP server instance with the provided settings.
    pub fn new(settings: TcpSettings) -> Result<TcpServerHandler<Created>, TcpHandlerError> {
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
            config: settings,
            id: Uuid::new_v4(),
            fd,
        };

        Ok(TcpServerHandler::<Created> {
            server,
            _state: PhantomData,
        })
    }
}

impl TcpServerHandler<Created> {
    /// Configure, bind, and place the server socket into listening mode.
    pub fn into_listening(self) -> Result<TcpServerHandler<Listening>, TcpHandlerError> {
        // Configure listener socket options before bind/listen.
        configure_listener_socket(self.server.fd)?;

        // Bind the socket to the specified host and port
        bind_socket(self.server.fd, &self.server.config.host, self.server.config.port)?;

        // Start listening for incoming connections
        listen_socket(self.server.fd, 128)?;

        Ok(TcpServerHandler::<Listening> {
            server: self.server,
            _state: PhantomData,
        })
    }
}

impl TcpServerHandler<Listening> {
    /// Accept clients in a loop and invoke the handler on a dedicated thread.
    pub fn run<H>(&self, handler: H) -> Result<(), TcpHandlerError> where H: Fn(TcpClient) + Send + Copy + 'static,
    {
        loop {
            let client_fd = accept_connection(self.server.fd)?; // blocking

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
    use std::net::TcpListener;

    use super::{Created, Listening, TcpServer, TcpServerHandler, TcpSettings};
    use crate::tcp_socket_handler::TcpHandlerError;

    fn pick_free_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("failed to pick free port");
        listener
            .local_addr()
            .expect("failed to read local address")
            .port()
    }

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
    fn initialize_transitions_created_to_listening() {
        let settings = TcpSettings {
            host: "127.0.0.1".to_string(),
            port: pick_free_port(),
        };

        let created_server: TcpServerHandler<Created> =
            TcpServer::new(settings).expect("expected Created server");

        let _listening_server: TcpServerHandler<Listening> = created_server
            .initialize()
            .expect("expected transition to Listening");
    }
}

