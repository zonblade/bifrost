use std::net::{TcpStream, SocketAddr};
use std::time::Duration;

#[derive(Debug)]
pub struct PortScanner {
    port: u16,
    open: bool,
}

pub async fn scan_ports(ip: &str, ports: &[u16])-> Vec<PortScanner> {
    let mut open_ports = vec![];
    for &port in ports {
        let address = format!("{}:{}", ip, port);
        let socket_addr: SocketAddr = address.parse().expect("Invalid address");
        println!("\rscanning port: {}\r", port);
        match TcpStream::connect_timeout(&socket_addr, Duration::from_secs(1)) {
            Ok(_) => {
                println!("Port {} is open", port);
                open_ports.push(PortScanner{
                    port,
                    open: true,
                });
            },
            Err(_) => continue,
        }
    }

    open_ports
}