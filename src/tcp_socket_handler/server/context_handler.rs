use std::sync::{Arc, RwLock};
use uuid::Uuid;
use crate::core::models::domain::tcp::{ContextHandler, TcpContext, TcpPeerConnection};
use crate::core::registry::register_error::RegistryError;

use crate::tcp_socket_handler::errors::TcpHandlerError;

impl ContextHandler {
    pub fn new(context: Arc<TcpContext>) -> Self {
        Self(context)
    }

    pub(crate) fn get_client(&self, client_id: Uuid) -> Result<Arc<RwLock<TcpPeerConnection>>, TcpHandlerError> {
        let client = self
            .0
            .registry
            .get_item(&client_id)
            .map_err(|_| TcpHandlerError::ContextLockError)?
            .ok_or_else(|| TcpHandlerError::ClientNotFound(format!("Client with ID {} not found", client_id)))?;
        Ok(client)
    }

    pub(crate) fn remove_client(&self, client_id: Uuid) -> Result<(), TcpHandlerError> {
        match self.0.registry.remove_item(&client_id) {
            Ok(()) => Ok(()),
            Err(RegistryError::KeyNotFound(_)) => Err(TcpHandlerError::ClientNotFound(format!("Client with ID {} not found", client_id))),
            Err(_) => Err(TcpHandlerError::ContextLockError),
        }
    }

    pub fn send_message_to_client(&self, client_id: Uuid, message: &[u8]) -> Result<(), TcpHandlerError> {
        let client = self
            .0
            .registry
            .get_item(&client_id)
            .map_err(|_| TcpHandlerError::ContextLockError)?
            .ok_or_else(|| TcpHandlerError::ClientNotFound(format!("Client with ID {} not found", client_id)))?;
        self.send_message(client, message)?;
        Ok(())
    }

    pub fn broadcast_message(&self, message: &[u8]) -> Result<(), TcpHandlerError> {
        let clients = self
            .0
            .registry
            .get_snapshot()
            .map_err(|_| TcpHandlerError::ContextLockError)?;

        for client in clients {
            self.send_message(client, message)?;
        }
        Ok(())
    }

    fn send_message(&self, client: Arc<RwLock<TcpPeerConnection>>, message: &[u8]) -> Result<(), TcpHandlerError> {
        let write_guard = client.write().map_err(|_| TcpHandlerError::ClientLockError)?;
        write_guard.write(message)?;
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::ContextHandler;
    use crate::core::models::domain::tcp::{TcpContext, TcpPeerConnection};
    use crate::core::registry::client_registry::ClientRegistry;
    use crate::tcp_socket_handler::errors::TcpHandlerError;
    use std::io::Read;
    use std::net::{TcpListener, TcpStream};
    use std::sync::Arc;
    use uuid::Uuid;

    #[cfg(unix)]
    use std::os::fd::IntoRawFd;
    #[cfg(windows)]
    use std::os::windows::io::IntoRawSocket;

    fn make_handler() -> ContextHandler {
        let context = TcpContext {
            server_id: Uuid::new_v4(),
            max_buffer_size: 1024,
            registry: ClientRegistry::new(),
        };

        ContextHandler::new(Arc::new(context))
    }

    fn connected_client_pair() -> (TcpPeerConnection, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind loopback listener");
        let addr = listener.local_addr().expect("failed to read listener address");

        let peer_stream = TcpStream::connect(addr).expect("failed to connect peer stream");
        let (server_stream, _) = listener.accept().expect("failed to accept server stream");

        #[cfg(unix)]
        let raw_socket = server_stream.into_raw_fd();
        #[cfg(windows)]
        let raw_socket = server_stream.into_raw_socket();

        let client = TcpPeerConnection {
            id: Uuid::new_v4(),
            fd: raw_socket,
        };

        (client, peer_stream)
    }

    #[test]
    fn get_client_missing_id_failure() {
        let handler = make_handler();

        let err = match handler.get_client(Uuid::new_v4()) {
            Ok(_) => panic!("missing client id should fail"),
            Err(err) => err,
        };

        assert!(matches!(err, TcpHandlerError::ClientNotFound(_)));
    }

    #[test]
    fn remove_client_missing_id_returns_client_not_found() {
        let handler = make_handler();

        let err = handler
            .remove_client(Uuid::new_v4())
            .expect_err("removing unknown id should fail");

        assert!(matches!(err, TcpHandlerError::ClientNotFound(_)));
    }

    #[test]
    fn remove_client_existing_id_success() {
        let handler = make_handler();
        let (client, _peer_stream) = connected_client_pair();
        let client_id = client.id;

        handler
            .0
            .registry
            .add_item(client_id, client)
            .expect("failed to seed client registry");

        handler.remove_client(client_id).expect("remove should succeed");

        assert_eq!(handler.0.registry.contains_key(&client_id).unwrap(), false);
    }

    #[test]
    fn send_message_to_client_delivers_bytes_success() {
        let handler = make_handler();
        let (client, mut peer_stream) = connected_client_pair();
        let client_id = client.id;

        handler
            .0
            .registry
            .add_item(client_id, client)
            .expect("failed to seed client registry");

        let payload = b"hello-client";
        handler
            .send_message_to_client(client_id, payload)
            .expect("send_message_to_client should succeed");

        let mut received = vec![0u8; payload.len()];
        peer_stream
            .read_exact(&mut received)
            .expect("peer should receive bytes sent to client");

        assert_eq!(received, payload);
    }

    #[test]
    fn broadcast_message_reaches_all_clients_success() {
        let handler = make_handler();
        let (client_a, mut peer_a) = connected_client_pair();
        let (client_b, mut peer_b) = connected_client_pair();
        let id_a = client_a.id;
        let id_b = client_b.id;

        handler
            .0
            .registry
            .add_item(id_a, client_a)
            .expect("failed to add first client");
        handler
            .0
            .registry
            .add_item(id_b, client_b)
            .expect("failed to add second client");

        let payload = b"hello-all";
        handler
            .broadcast_message(payload)
            .expect("broadcast_message should succeed");

        let mut received_a = vec![0u8; payload.len()];
        let mut received_b = vec![0u8; payload.len()];

        peer_a
            .read_exact(&mut received_a)
            .expect("first peer should receive broadcast");
        peer_b
            .read_exact(&mut received_b)
            .expect("second peer should receive broadcast");

        assert_eq!(received_a, payload);
        assert_eq!(received_b, payload);
    }
}