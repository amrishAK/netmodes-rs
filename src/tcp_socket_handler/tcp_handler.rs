use uuid::Uuid;
use super::raw_socket::*;
use super::raw_socket::errors::SocketError;
use super::errors::TcpHandlerError;
use std::thread::spawn;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpServerState {
    Created,
    Bound,
    Listening,
    Closed,
}

pub struct TcpSettings {
    pub host: String,
    pub port: u16,
}

pub struct TcpClient {
    pub id: String,
    fd: Socket,
}

pub struct TcpServer {
    pub settings: TcpSettings,
    pub id: String,
    fd: Socket,
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
    pub fn read(&self, buffer: &mut [u8]) -> Result<usize, TcpHandlerError> {
        let bytes_received = receive_data(self.fd, buffer)?;
        Ok(bytes_received)
    }

    pub fn write(&self, data: &[u8]) -> Result<(), TcpHandlerError> 
    {
        let bytes_sent = send_data(self.fd, data)?; // SocketError -> TcpHandlerError via From

        if bytes_sent != data.len() {
            return Err(TcpHandlerError::TcpClientWriteError(
                SocketError::SendDataError(std::io::Error::new(
                    std::io::ErrorKind::WriteZero,
                    format!("Failed to send all data: sent {bytes_sent} of {}", data.len()),
                )),
            ));
        }
        
        Ok(())
    }

    pub fn write_partial(&self, data: &[u8]) -> Result<usize, TcpHandlerError> 
    {
        let bytes_sent = send_data(self.fd, data)?; // SocketError -> TcpHandlerError via From
        Ok(bytes_sent)
    }
}

impl TcpServer {

    pub fn new(settings: TcpSettings) -> Result<Self, TcpHandlerError> {
        
        // Validate port and host
        if settings.port == 0 {
            return Err(TcpHandlerError::TcpPortValidationError(format!("Invalid port number: {}", settings.port)));
        }

        if settings.host.is_empty() {
            return Err(TcpHandlerError::TcpHostValidationError(format!("Invalid host: {}", settings.host)));
        }
        
        // Create the TCP socket
        let fd = create_tcp_socket()?;
        
        // Return the TcpServer instance
        let server = TcpServer {
            settings,
            id: format!("tcp-server-{}", Uuid::new_v4()),
            fd,
            state: TcpServerState::Created,
        };

        Ok(server)
    }

    pub fn initialize(&mut self) -> Result<(), TcpHandlerError> {
        
        if self.state != TcpServerState::Created
        {
            return Err(TcpHandlerError::TcpStateError(format!("Invalid state for initialization: {:?}", self.state)));
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
    
    
    pub fn run<H>(&self, handler: H) -> Result<(), TcpHandlerError> where H: Fn(TcpClient) + Send + Copy + 'static,
    {
        if self.state != TcpServerState::Listening {
            return Err(TcpHandlerError::TcpStateError(format!("Invalid state for running server: {:?}", self.state)));
        }

        loop {
            let client_fd = accept_connection(self.fd)?; // blocking

            let client = TcpClient {
                id: format!("tcp-client-{}", Uuid::new_v4()),
                fd: client_fd,
            };

            // user-provided connection handler
            spawn(move || {
                handler(client);
            });
        }
    }
}

