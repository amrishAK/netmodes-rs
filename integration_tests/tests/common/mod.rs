use std::net::TcpListener;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use netmodes_rs::core::models::domain::tcp::{ContextHandler, OnMessageHandler, TcpClientSession};
use netmodes_rs::tcp_socket_handler::{
    Disconnected,
    TcpClientConfiguration,
    TcpClientHandler,
    TcpServer,
    TcpServerConfiguration,
};

pub fn pick_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to pick free port");
    listener
        .local_addr()
        .expect("failed to read local address")
        .port()
}

pub fn wait_for_server(host: &str, port: u16, timeout: Duration) {
    let start = Instant::now();

    while start.elapsed() < timeout {
        let disconnected = TcpClientHandler::<Disconnected>::new(TcpClientConfiguration {
            host: host.to_string(),
            port,
            max_buffer_size: 1024,
        })
        .expect("failed to create readiness client");

        if disconnected.connect().is_ok() {
            return;
        }

        thread::sleep(Duration::from_millis(25));
    }

    panic!("server did not start on {host}:{port} in time");
}

pub fn echo_message_handler() -> OnMessageHandler {
    Arc::new(|client: &TcpClientSession, _context: ContextHandler, data: &[u8]| {
        let _ = client.reply(data);
    })
}

pub fn start_server(host: &str, port: u16, max_buffer_size: usize, handler: OnMessageHandler) {
    let host = host.to_string();

    thread::spawn(move || {
        let settings = TcpServerConfiguration {
            host,
            port,
            max_buffer_size,
        };

        let server = TcpServer::new(settings).expect("failed to create tcp server");
        let server = server
            .into_listening()
            .expect("failed to transition server to listening state");

        let _ = server.run(handler);
    });
}