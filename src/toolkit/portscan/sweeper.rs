use std::net::{TcpStream, SocketAddr};
use std::time::Duration;

pub fn scan_ports(ip: &str, ports: &[u16]) {
    for &port in ports {
        let address = format!("{}:{}", ip, port);
        let socket_addr: SocketAddr = address.parse().expect("Invalid address");

        match TcpStream::connect_timeout(&socket_addr, Duration::from_secs(1)) {
            Ok(_) => println!("Port {} is open", port),
            Err(_) => println!("Port {} is closed", port),
        }
    }
}