use std::marker::PhantomData;

use uuid::Uuid;

use crate::core::models::domain::state_type::{Connected, Disconnected};
use crate::core::models::domain::tcp::{
	TcpClient,
	TcpClientConfiguration,
	TcpClientHandler,
};
use crate::core::socket::{close_socket, connect_socket, create_tcp_socket, receive_data, send_data};
use crate::tcp_socket_handler::errors::TcpHandlerError;

impl TcpClient {
	pub fn read(&self, buffer: &mut [u8]) -> Result<usize, TcpHandlerError> {
		receive_data(self.fd, buffer).map_err(Into::into)
	}

	pub fn write(&self, data: &[u8]) -> Result<(), TcpHandlerError> {
		let bytes_sent = send_data(self.fd, data)?;

		if bytes_sent != data.len() {
			return Err(TcpHandlerError::PartialWrite {
				sent: bytes_sent,
				total: data.len(),
			});
		}

		Ok(())
	}
}

impl Drop for TcpClient {
	fn drop(&mut self) {
		_ = close_socket(self.fd);
	}
}

impl<State> TcpClientHandler<State> {
	pub fn id(&self) -> Uuid {
		self.client.id
	}

	pub fn config(&self) -> &TcpClientConfiguration {
		&self.client.config
	}
}

impl TcpClientHandler<Disconnected> {
	/// Create a disconnected TCP client with a socket ready to connect.
	pub fn new(config: TcpClientConfiguration) -> Result<Self, TcpHandlerError> {
		if config.port == 0 {
			return Err(TcpHandlerError::InvalidPort(format!(
				"Invalid port number: {}",
				config.port
			)));
		}

		if config.host.is_empty() {
			return Err(TcpHandlerError::InvalidHost(format!("Invalid host: {}", config.host)));
		}

		let fd = create_tcp_socket()?;

		Ok(Self {
			client: TcpClient {
				config,
				id: Uuid::new_v4(),
				fd,
			},
			_state: PhantomData,
		})
	}

	/// Connect this client to the configured remote endpoint.
	pub fn connect(self) -> Result<TcpClientHandler<Connected>, TcpHandlerError> {
		connect_socket(self.client.fd, &self.client.config.host, self.client.config.port)?;

		Ok(TcpClientHandler {
			client: self.client,
			_state: PhantomData,
		})
	}
}

impl TcpClientHandler<Connected> {
	/// Send a full payload to the connected remote endpoint.
	pub fn send(&self, data: &[u8]) -> Result<(), TcpHandlerError> {
		self.client.write(data)
	}

	/// Receive a single payload chunk from the remote endpoint.
	pub fn receive(&self) -> Result<Vec<u8>, TcpHandlerError> {
		let mut buffer = vec![0u8; self.client.config.max_buffer_size];
		let bytes_read = self.client.read(&mut buffer)?;

		if bytes_read == 0 {
			return Err(TcpHandlerError::ConnectionClosed);
		}

		buffer.truncate(bytes_read);
		Ok(buffer)
	}
}

#[cfg(test)]
mod tests {
	use std::io::{Read, Write};
	use std::net::TcpListener;
	use std::thread;

	use super::*;

	fn loopback_client_config(port: u16) -> TcpClientConfiguration {
		TcpClientConfiguration {
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
	fn connect_send_receive_success() {
		let port = pick_free_port();
		let listener = TcpListener::bind(("127.0.0.1", port)).expect("failed to bind loopback listener");

		let server = thread::spawn(move || {
			let (mut stream, _) = listener.accept().expect("failed to accept client connection");

			let mut incoming = [0u8; 32];
			let bytes_read = stream.read(&mut incoming).expect("server failed to read payload");
			assert_eq!(&incoming[..bytes_read], b"ping");

			stream.write_all(b"pong").expect("server failed to write response");
		});

		let disconnected = TcpClientHandler::<Disconnected>::new(loopback_client_config(port))
			.expect("failed to create disconnected client");
		let connected = disconnected.connect().expect("failed to connect client");

		connected.send(b"ping").expect("client failed to send payload");
		let response = connected.receive().expect("client failed to receive payload");

		assert_eq!(response, b"pong");
		server.join().expect("server thread should complete cleanly");
	}
}
