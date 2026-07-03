mod common;

use std::thread;
use std::time::Duration;

use thugal_net::tcp_socket_handler::{
    Disconnected,
    TcpClientConfiguration,
    TcpClientHandler,
};

use common::{echo_message_handler, pick_free_port, start_server, wait_for_server};

#[test]
fn high_sequential_messages_single_connection_success() {
    let host = "127.0.0.1";
    let port = pick_free_port();
    start_server(host, port, 2048, echo_message_handler());
    wait_for_server(host, port, Duration::from_secs(3));

    let disconnected = TcpClientHandler::<Disconnected>::new(TcpClientConfiguration {
        host: host.to_string(),
        port,
        max_buffer_size: 2048,
    })
    .expect("failed to construct disconnected client");

    let connected = disconnected
        .connect()
        .expect("failed to connect client");

    for idx in 0..20 {
        let payload = format!("message-{idx}").into_bytes();
        connected
            .send(&payload)
            .expect("failed to send message to server");

        let echoed = connected
            .receive()
            .expect("failed to receive echoed payload");

        assert_eq!(echoed, payload);
    }
}

#[test]
fn high_concurrent_clients_round_trip_success() {
    let host = "127.0.0.1";
    let port = pick_free_port();
    start_server(host, port, 2048, echo_message_handler());
    wait_for_server(host, port, Duration::from_secs(3));

    let mut workers = Vec::new();

    for idx in 0..10 {
        let host = host.to_string();
        workers.push(thread::spawn(move || {
            let disconnected = TcpClientHandler::<Disconnected>::new(TcpClientConfiguration {
                host,
                port,
                max_buffer_size: 2048,
            })
            .expect("failed to construct disconnected client");

            let connected = disconnected
                .connect()
                .expect("failed to connect client");

            let payload = format!("client-{idx}-payload").into_bytes();
            connected
                .send(&payload)
                .expect("failed to send client payload");

            let echoed = connected
                .receive()
                .expect("failed to receive echoed payload");

            assert_eq!(echoed, payload);
        }));
    }

    for worker in workers {
        worker
            .join()
            .expect("client worker thread should complete cleanly");
    }
}

#[test]
fn high_large_payload_round_trip_success() {
    let host = "127.0.0.1";
    let port = pick_free_port();
    start_server(host, port, 65_536, echo_message_handler());
    wait_for_server(host, port, Duration::from_secs(3));

    let disconnected = TcpClientHandler::<Disconnected>::new(TcpClientConfiguration {
        host: host.to_string(),
        port,
        max_buffer_size: 65_536,
    })
    .expect("failed to construct disconnected client");

    let connected = disconnected
        .connect()
        .expect("failed to connect client");

    let payload = vec![b'a'; 32_768];
    connected
        .send(&payload)
        .expect("failed to send large payload");

    let echoed = connected
        .receive()
        .expect("failed to receive echoed large payload");

    assert_eq!(echoed, payload);
}