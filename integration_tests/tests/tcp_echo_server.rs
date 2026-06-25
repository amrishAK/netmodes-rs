use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

use netmodes_rs::tcp_socket_handler::{TcpClient, TcpHandlerError, TcpServer, TcpSettings};

fn echo_client_handler(client: TcpClient) {
    let mut buffer = [0_u8; 1024];

    loop {
        match client.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                if client.write(&buffer[..n]).is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}

fn pick_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to pick free port");
    listener
        .local_addr()
        .expect("failed to read local address")
        .port()
}

fn wait_for_server(port: u16, timeout: Duration) {
    let start = Instant::now();

    while start.elapsed() < timeout {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(25));
    }

    panic!("server did not start on port {port} in time");
}

#[test]
fn zero_port_failure() {
    let settings = TcpSettings {
        host: "127.0.0.1".to_string(),
        port: 0,
    };

    let err = match TcpServer::new(settings) {
        Ok(_) => panic!("expected invalid port error"),
        Err(err) => err,
    };
    assert!(matches!(err, TcpHandlerError::InvalidPort(_)));
}

#[test]
fn empty_host_failure() {
    let settings = TcpSettings {
        host: "".to_string(),
        port: 8080,
    };

    let err = match TcpServer::new(settings) {
        Ok(_) => panic!("expected invalid host error"),
        Err(err) => err,
    };
    assert!(matches!(err, TcpHandlerError::InvalidHost(_)));
}

#[test]
fn initialize_transitions_to_listening_success() {
    let settings = TcpSettings {
        host: "127.0.0.1".to_string(),
        port: pick_free_port(),
    };

    let server = TcpServer::new(settings).expect("failed to create tcp server");
    let listening_server = server
        .into_listening()
        .expect("into_listening should transition to listening");

    // Typestate prevents calling initialize a second time at compile time.
    drop(listening_server);
}

#[test]
fn echo_round_trip_success() {
    let port = pick_free_port();

    thread::spawn(move || {
        let settings = TcpSettings {
            host: "127.0.0.1".to_string(),
            port,
        };

        let server = TcpServer::new(settings).expect("failed to create tcp server");
        let server = server
            .into_listening()
            .expect("failed to transition server to listening state");

        let _ = server.run(echo_client_handler);
    });

    wait_for_server(port, Duration::from_secs(3));

    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("failed to connect to echo server");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("failed to set read timeout");

    let payload = b"echo-check";
    stream
        .write_all(payload)
        .expect("failed to write payload to server");

    let mut echoed = vec![0_u8; payload.len()];
    stream
        .read_exact(&mut echoed)
        .expect("failed to read echoed payload");

    assert_eq!(echoed, payload);
}
