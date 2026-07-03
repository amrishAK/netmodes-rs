mod common;

use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use thugal_net::core::socket::SocketError;
use thugal_net::core::models::domain::tcp::{ContextHandler, OnMessageHandler, TcpClientSession};
use thugal_net::tcp_socket_handler::{
    Disconnected,
    TcpClientConfiguration,
    TcpClientHandler,
    TcpHandlerError,
    TcpServer,
    TcpServerConfiguration,
};

use common::{echo_message_handler, pick_free_port, start_server, wait_for_server};

#[test]
fn server_new_with_zero_port_failure() {
    let settings = TcpServerConfiguration {
        host: "127.0.0.1".to_string(),
        port: 0,
        max_buffer_size: 1024,
    };

    let err = match TcpServer::new(settings) {
        Ok(_) => panic!("expected invalid port error"),
        Err(err) => err,
    };

    assert!(matches!(err, TcpHandlerError::InvalidPort(_)));
}

#[test]
fn server_new_with_empty_host_failure() {
    let settings = TcpServerConfiguration {
        host: "".to_string(),
        port: 8080,
        max_buffer_size: 1024,
    };

    let err = match TcpServer::new(settings) {
        Ok(_) => panic!("expected invalid host error"),
        Err(err) => err,
    };

    assert!(matches!(err, TcpHandlerError::InvalidHost(_)));
}

#[test]
fn server_into_listening_transition_success() {
    let settings = TcpServerConfiguration {
        host: "127.0.0.1".to_string(),
        port: pick_free_port(),
        max_buffer_size: 1024,
    };

    let created = TcpServer::new(settings).expect("failed to create tcp server");
    let listening = created
        .into_listening()
        .expect("expected transition to listening");

    drop(listening);
}

#[test]
fn server_bind_conflicting_port_failure() {
    let occupied = TcpListener::bind("127.0.0.1:0").expect("failed to occupy test port");
    let port = occupied
        .local_addr()
        .expect("failed to read occupied local address")
        .port();

    let settings = TcpServerConfiguration {
        host: "127.0.0.1".to_string(),
        port,
        max_buffer_size: 1024,
    };

    let created = TcpServer::new(settings).expect("failed to create tcp server");

    let err = match created.into_listening() {
        Ok(_) => panic!("expected bind failure on occupied port"),
        Err(err) => err,
    };

    assert!(matches!(err, TcpHandlerError::Socket(SocketError::BindSocket(_))));
}

#[test]
fn server_run_invokes_message_handler_success() {
    let host = "127.0.0.1";
    let port = pick_free_port();
    let calls = Arc::new(AtomicUsize::new(0));
    let calls_for_handler = Arc::clone(&calls);

    let handler: OnMessageHandler = Arc::new(move |client: &TcpClientSession, _context: ContextHandler, data: &[u8]| {
        calls_for_handler.fetch_add(1, Ordering::SeqCst);
        let _ = client.reply(data);
    });

    start_server(host, port, 1024, handler);
    wait_for_server(host, port, Duration::from_secs(3));

    let disconnected = TcpClientHandler::<Disconnected>::new(TcpClientConfiguration {
        host: host.to_string(),
        port,
        max_buffer_size: 1024,
    })
    .expect("failed to construct disconnected tcp client");

    let connected = disconnected
        .connect()
        .expect("failed to connect tcp client handler");

    connected
        .send(b"trigger-handler")
        .expect("failed to send payload to server");

    let _ = connected
        .receive()
        .expect("failed to receive handler echo response");

    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(2) {
        if calls.load(Ordering::SeqCst) > 0 {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    assert!(calls.load(Ordering::SeqCst) > 0);
}

#[test]
fn server_echo_round_trip_success() {
    let host = "127.0.0.1";
    let port = pick_free_port();
    start_server(host, port, 1024, echo_message_handler());
    wait_for_server(host, port, Duration::from_secs(3));

    let disconnected = TcpClientHandler::<Disconnected>::new(TcpClientConfiguration {
        host: host.to_string(),
        port,
        max_buffer_size: 1024,
    })
    .expect("failed to construct disconnected tcp client");

    let connected = disconnected
        .connect()
        .expect("failed to connect tcp client handler");

    let payload = b"server-echo-contract";
    connected
        .send(payload)
        .expect("failed to send payload to server");

    let echoed = connected
        .receive()
        .expect("failed to receive echoed payload");

    assert_eq!(echoed, payload);
}