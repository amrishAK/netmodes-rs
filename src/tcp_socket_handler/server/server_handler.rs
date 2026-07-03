use std::marker::PhantomData;
use std::sync::Arc;
use std::thread::spawn;
use uuid::Uuid;

pub use crate::core::models::domain::state_type::{Created, Listening, Unconfigured};
use crate::core::registry::client_registry::ClientRegistry;
use crate::core::socket::*;
pub use crate::core::models::domain::tcp::*;

use crate::tcp_socket_handler::server::run_client_session;
use crate::tcp_socket_handler::errors::TcpHandlerError;


impl<State> TcpServerHandler<State> {
    pub fn config(&self) -> &TcpServerConfiguration {
        &self.server.config
    }

    pub fn id(&self) -> Uuid {
        self.server.id
    }
}

impl Drop for TcpServer{
    fn drop(&mut self) {
        // Close the server socket when the TcpServer instance is dropped
        if let Err(e) = close_socket(self.fd) {
            eprintln!("Failed to close server socket: {:?}", e);
        }
    }
}


impl TcpServer {
    /// Create a new TCP server instance with the provided settings.
    pub fn new(settings: TcpServerConfiguration) -> Result<TcpServerHandler<Created>, TcpHandlerError> {
        // Validate port and host
        if settings.port == 0 {
            return Err(TcpHandlerError::InvalidPort(format!("Invalid port number: {}", settings.port)));
        }

        if settings.host.is_empty() {
            return Err(TcpHandlerError::InvalidHost(format!("Invalid host: {}", settings.host)));
        }
        
        // Create the TCP socket
        let fd = create_tcp_socket()?;
        
        // Create the TcpServerHandler in the Created state
        let server_handler = Self::get_tcp_server_handler(fd, settings)?;

        Ok(server_handler)
    }

    fn get_tcp_server_handler(
        server_fd: Socket,
        settings: TcpServerConfiguration,
    ) -> Result<TcpServerHandler<Created>, TcpHandlerError> {
        
        // Extract buffer size before settings is moved
        let buffer_size = settings.max_buffer_size;
        
        // Create the TcpServer instance
        let server = TcpServer {
            config: settings,
            id: Uuid::new_v4(),
            fd: server_fd
        };

        // Create the TcpContext with a new ClientRegistry
        let context = TcpContext {
            server_id: server.id,
            registry: ClientRegistry::new(),
            max_buffer_size: buffer_size,
        };

        // Return the TcpServerHandler in the Created state
        Ok(TcpServerHandler::<Created> {
            server: server,
            context: Arc::new(context),
            _state: PhantomData
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
            context: self.context,
            _state: PhantomData,
        })
    }
}

impl TcpServerHandler<Listening> {
    /// Accept clients in a loop and invoke the handler on a dedicated thread.
    pub fn run(&self, message_handler: OnMessageHandler) -> Result<(), TcpHandlerError> {
        loop {
            
            let client_fd = match accept_connection(self.server.fd) {
                Ok(fd) => fd,
                Err(err) => {
                    eprintln!("Failed to accept client connection: {:?}", err);
                    std::thread::sleep(std::time::Duration::from_millis(50)); // Sleep briefly before retrying
                    continue; // Continue accepting new clients even if one fails
                }
            };
            
            let client = TcpPeerConnection {
                id: Uuid::new_v4(),
                fd: client_fd,
            };

            // Register the client in the context's registry
            let client_id = client.id;
            self.context
                .registry
                .add_item(client_id, client)
                .map_err(|err| {
                    TcpHandlerError::RegistrationError(format!(
                        "failed to register client {client_id}: {err}"
                    ))
                })?;

            // user-provided connection handler
            let context = ContextHandler(self.context.clone());
            let message_handler = Arc::clone(&message_handler);
            spawn(move || {
                run_client_session(client_id, context, message_handler);
            });
        }
    }
}

#[cfg(test)]
mod tcp_handlers_tests {
    use std::net::TcpListener;
    use uuid::Uuid;

    use super::{Created, Listening, TcpServer, TcpServerConfiguration, TcpServerHandler};
    use crate::tcp_socket_handler::TcpHandlerError;

    fn loopback_settings(port: u16) -> TcpServerConfiguration {
        TcpServerConfiguration {
            host: "127.0.0.1".to_string(),
            port,
            max_buffer_size: 1024,
        }
    }

    fn pick_free_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("failed to pick free port");
        listener
            .local_addr()
            .expect("failed to read local address")
            .port()
    }

    #[test]
    fn new_with_zero_port_returns_invalid_port_failure() {
        let settings = loopback_settings(0);

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
        let settings = TcpServerConfiguration { host: "".to_string(), port: 8080, max_buffer_size: 1024 };

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
    fn new_returns_created_handler_with_config_and_id_success() {
        let settings = loopback_settings(pick_free_port());

        let created_server: TcpServerHandler<Created> =
            TcpServer::new(settings).expect("expected Created server");

        assert_eq!(created_server.config().host, "127.0.0.1");
        assert_ne!(created_server.config().port, 0);
        assert_ne!(created_server.id(), Uuid::nil());
    }

    #[test]
    fn into_listening_preserves_server_identity_success() {
        let settings = loopback_settings(pick_free_port());

        let created_server: TcpServerHandler<Created> =
            TcpServer::new(settings).expect("expected Created server");
        let created_id = created_server.id();
        let created_host = created_server.config().host.clone();
        let created_port = created_server.config().port;

        let _listening_server: TcpServerHandler<Listening> = created_server.into_listening()
            .expect("expected transition to Listening");

        assert_eq!(_listening_server.id(), created_id);
        assert_eq!(_listening_server.config().host, created_host);
        assert_eq!(_listening_server.config().port, created_port);
    }
}

