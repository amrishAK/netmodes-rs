use netmodes_rs::tcp_socket_handler::*;

fn tcp_client_handler(client: TcpClient) {
    let mut buffer = [0; 512];

    loop {
        match client.read(&mut buffer) {
            Ok(0) => break, // EOF
            Ok(n) => {
                client.write(&buffer[..n]).unwrap();
                println!("Received: {}", String::from_utf8_lossy(&buffer[..n]));
            }
            Err(e) => {
                println!("Error occurred while receiving message from client: {}", e);
                break;
            }
        }
    }
}

fn main() {
    let server_settings = TcpSettings {
        host: "0.0.0.0".to_string(),
        port: 5500,
    };

    let tcp_server_handler = TcpServer::new(server_settings).expect("Failed to create TCP server");
    println!(
        "TCP server is listening on {}:{}",
        tcp_server_handler.config().host, tcp_server_handler.config().port
    );

    let tcp_server_handler = tcp_server_handler.into_listening().expect("Failed to initialize TCP server");
    println!("TCP server has been initialized and is accepting connections");

    tcp_server_handler.run(tcp_client_handler).expect("Failed to run TCP server");
}