use std::sync::Arc;

use thugal_net::core::models::domain::tcp::{ContextHandler, OnMessageHandler, TcpClientSession};
use thugal_net::tcp_socket_handler::*;



fn main() {
    let server_settings = TcpServerConfiguration {
        host: "0.0.0.0".to_string(),
        port: 5500,
        max_buffer_size: 8192,
    };

    let tcp_server_handler = TcpServer::new(server_settings).expect("Failed to create TCP server");
    println!(
        "TCP server is listening on {}:{}",
        tcp_server_handler.config().host, tcp_server_handler.config().port
    );

    let tcp_server_handler = tcp_server_handler.into_listening().expect("Failed to initialize TCP server");
    println!("TCP server has been initialized and is accepting connections");

    let on_message_handler: OnMessageHandler = Arc::new(|client_handler: &TcpClientSession, _context_handler: ContextHandler, data: &[u8]| {
        client_handler.reply(data).expect("Failed to send reply to client");
        match client_handler.get_client_id() {
            Ok(client_id) => {
                println!("Received message from client {}: {}", client_id, String::from_utf8_lossy(data));
            }
            Err(err) => {
                eprintln!("Failed to get client ID: {:?}", err);
            }
        }
    });

    tcp_server_handler
        .run(on_message_handler)
        .expect("Failed to run TCP server");
}