mod common;

use std::time::Duration;

use netmodes_rs::tcp_socket_handler::{
    Disconnected,
    TcpClientConfiguration,
    TcpClientHandler,
    TcpHandlerError,
};

use common::{echo_message_handler, pick_free_port, start_server, wait_for_server};

#[test]
fn client_new_with_zero_port_failure() {
    let err = match TcpClientHandler::<Disconnected>::new(TcpClientConfiguration {
        host: "127.0.0.1".to_string(),
        port: 0,
        max_buffer_size: 1024,
    }) {
        Ok(_) => panic!("port 0 should fail for client configuration"),
        Err(err) => err,
    };

    assert!(matches!(err, TcpHandlerError::InvalidPort(_)));
}

#[test]
fn client_new_with_empty_host_failure() {
    let err = match TcpClientHandler::<Disconnected>::new(TcpClientConfiguration {
        host: "".to_string(),
        port: 8080,
        max_buffer_size: 1024,
    }) {
        Ok(_) => panic!("empty host should fail for client configuration"),
        Err(err) => err,
    };

    assert!(matches!(err, TcpHandlerError::InvalidHost(_)));
}

#[test]
fn client_connect_without_listening_server_failure() {
    let disconnected = TcpClientHandler::<Disconnected>::new(TcpClientConfiguration {
        host: "127.0.0.1".to_string(),
        port: pick_free_port(),
        max_buffer_size: 1024,
    })
    .expect("expected disconnected client construction to succeed");

    let connect_result = disconnected.connect();
    assert!(connect_result.is_err());
}

#[test]
fn client_send_receive_echo_success() {
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

    let payload = b"client-echo-message";
    connected
        .send(payload)
        .expect("failed to send payload from client handler");

    let echoed = connected
        .receive()
        .expect("failed to receive echoed payload");

    assert_eq!(echoed, payload);
}

#[test]
fn client_receive_respects_max_buffer_size_edge_success() {
    let host = "127.0.0.1";
    let port = pick_free_port();
    start_server(host, port, 4096, echo_message_handler());
    wait_for_server(host, port, Duration::from_secs(3));

    let disconnected = TcpClientHandler::<Disconnected>::new(TcpClientConfiguration {
        host: host.to_string(),
        port,
        max_buffer_size: 8,
    })
    .expect("failed to construct disconnected tcp client");

    let connected = disconnected
        .connect()
        .expect("failed to connect tcp client handler");

    let payload = b"1234567890ABCDEF";
    connected
        .send(payload)
        .expect("failed to send payload from client handler");

    let first_chunk = connected
        .receive()
        .expect("failed to receive first payload chunk");

    assert_eq!(first_chunk.len(), 8);
    assert_eq!(&first_chunk[..], &payload[..8]);
}