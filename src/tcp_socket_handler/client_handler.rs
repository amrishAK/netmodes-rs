use crate::core::socket::*;
use crate::core::models::domain::tcp::{ClientHandler, ContextHandler, OnMessageHandler, TcpClient};
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use super::errors::TcpHandlerError;


impl ClientHandler {
    
    fn new(client: Arc<RwLock<TcpClient>>) -> Self {
        Self(client)
    }

    /// Get the unique ID of this client.
    pub fn get_client_id(&self) -> Result<Uuid, TcpHandlerError> {
        let client = self.0.read().map_err(|_| TcpHandlerError::ClientLockError)?;
        Ok(client.id)
    }

    /// Send data to this client.
    ///
    /// This method will fail if the lock is poisoned or if the socket write fails.
    pub fn reply(&self,  data: &[u8]) -> Result<(), TcpHandlerError> {
        let client = self.0.read().map_err(|_| TcpHandlerError::ClientLockError)?;
        client.write(data)?;
        println!("Echoed back: {}", String::from_utf8_lossy(data));
        Ok(())
    }
}

impl TcpClient {
    /// Read bytes from the client socket into the provided buffer.
    pub fn read(&self, buffer: &mut [u8]) -> Result<usize, TcpHandlerError> {
        receive_data(self.fd, buffer).map_err(Into::into)
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

impl Drop for TcpClient {
    fn drop(&mut self) {
        _ = close_socket(self.fd);
    }
}

fn get_read_lock(client: &Arc<RwLock<TcpClient>>) -> Result<std::sync::RwLockReadGuard<'_, TcpClient>, TcpHandlerError> {
    client.read().map_err(|_| TcpHandlerError::ClientLockError)
}

pub(crate) fn tcp_client_handler(client_id: Uuid, tcp_context: ContextHandler, message_handler: OnMessageHandler) {
    
    let client : Arc<RwLock<TcpClient>> = match tcp_context.get_client(client_id) {
        Ok(handler) => handler,
        Err(err) => {
            eprintln!("Failed to retrieve client handler for client {}: {:?}", client_id, err);
            return;
        }
    }; 

    let buffer_size = tcp_context.0.max_buffer_size;
    let mut buffer = vec![0u8; buffer_size];
    let handler = ClientHandler::new(client.clone());

    loop {
        let bytes_read = {
            let read_lock = match get_read_lock(&client) {
                Ok(lock) => lock,
                Err(err) => {
                    eprintln!("Failed to acquire read lock on TcpClient for client {}: {:?}", client_id, err);
                    break;
                }
            };

            match read_lock.read(&mut buffer) {
                Ok(0) => {
                    println!("Client {} disconnected", client_id);
                    break;
                }
                Ok(bytes_read) => bytes_read,
                Err(err) => {
                    eprintln!("Error reading from client {}: {:?}", client_id, err);
                    break;
                }
            }
        };

        let data = &buffer[..bytes_read];

        // Call the user-provided message handler after releasing the read lock.
        message_handler(&handler, tcp_context.clone(), data);
    }

    // Remove the client from the registry when done
    if let Err(err) = tcp_context.remove_client(client_id) {
        eprintln!("Failed to remove client {} from registry: {:?}", client_id, err);
    } else {
        println!("Client {} removed from registry", client_id);
    }
}

#[cfg(test)]
mod tests {
    use super::{tcp_client_handler, ClientHandler, ContextHandler, OnMessageHandler, TcpClient};
    use crate::core::models::domain::tcp::TcpContext;
    use crate::core::registry::client_registry::ClientRegistry;
    use std::io::Write;
    use std::net::{Shutdown, TcpListener, TcpStream};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex, RwLock};
    use std::thread;
    use uuid::Uuid;

    #[cfg(unix)]
    use std::os::fd::IntoRawFd;
    #[cfg(windows)]
    use std::os::windows::io::IntoRawSocket;

    fn make_context_handler() -> ContextHandler {
        let context = TcpContext {
            server_id: Uuid::new_v4(),
            registry: ClientRegistry::new(),
            max_buffer_size: 1024,
        };

        ContextHandler::new(Arc::new(context))
    }

    fn connected_client_pair() -> (TcpClient, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind listener");
        let addr = listener.local_addr().expect("failed to read listener address");

        let peer_stream = TcpStream::connect(addr).expect("failed to connect peer stream");
        let (server_stream, _) = listener.accept().expect("failed to accept server stream");

        #[cfg(unix)]
        let raw_socket = server_stream.into_raw_fd();
        #[cfg(windows)]
        let raw_socket = server_stream.into_raw_socket();

        let client = TcpClient {
            id: Uuid::new_v4(),
            fd: raw_socket,
        };

        (client, peer_stream)
    }

    #[test]
    fn get_client_id_returns_underlying_client_id_success() {
        let (client, _peer_stream) = connected_client_pair();
        let expected_id = client.id;
        let handler = ClientHandler::new(Arc::new(RwLock::new(client)));

        let actual_id = handler.get_client_id().expect("get_client_id should succeed");

        assert_eq!(actual_id, expected_id);
    }

    #[test]
    fn reply_writes_data_to_peer_success() {
        let (client, mut peer_stream) = connected_client_pair();
        let handler = ClientHandler::new(Arc::new(RwLock::new(client)));
        let payload = b"reply-message";

        handler.reply(payload).expect("reply should succeed");

        let mut received = vec![0u8; payload.len()];
        std::io::Read::read_exact(&mut peer_stream, &mut received)
            .expect("peer should receive replied payload");
        assert_eq!(received, payload);
    }

    #[test]
    fn tcp_client_handler_forwards_message_and_removes_client_success() {
        let tcp_context = make_context_handler();
        let (client, mut peer_stream) = connected_client_pair();
        let client_id = client.id;

        tcp_context
            .0
            .registry
            .add_item(client_id, client)
            .expect("failed to seed registry");

        let observed = Arc::new(Mutex::new(Vec::<u8>::new()));
        let observed_ref = Arc::clone(&observed);
        let message_handler: OnMessageHandler = Arc::new(move |_client_handler, _ctx, data| {
            observed_ref
                .lock()
                .expect("message buffer lock poisoned")
                .extend_from_slice(data);
        });

        let context_for_thread = tcp_context.clone();
        let handle = thread::spawn(move || {
            tcp_client_handler(client_id, context_for_thread, message_handler);
        });

        let payload = b"hello-from-peer";
        peer_stream
            .write_all(payload)
            .expect("failed to write payload to client socket");
        peer_stream
            .shutdown(Shutdown::Write)
            .expect("failed to shutdown write half");

        handle.join().expect("tcp_client_handler thread should exit");

        let recorded = observed.lock().expect("message buffer lock poisoned").clone();
        assert_eq!(recorded, payload);
        assert_eq!(tcp_context.0.registry.contains_key(&client_id).unwrap(), false);
    }

    #[test]
    fn tcp_client_handler_missing_client_does_not_invoke_handler_success() {
        let tcp_context = make_context_handler();
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_ref = Arc::clone(&call_count);

        let message_handler: OnMessageHandler = Arc::new(move |_client_handler, _ctx, _data| {
            call_count_ref.fetch_add(1, Ordering::SeqCst);
        });

        tcp_client_handler(Uuid::new_v4(), tcp_context, message_handler);

        assert_eq!(call_count.load(Ordering::SeqCst), 0);
    }
}
