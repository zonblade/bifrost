use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::time::Duration;

pub fn tcp_banner(ip: &str, port: u32) -> io::Result<String> {
    let address = format!("{}:{}", ip, port);
    let socket_addr: SocketAddr = address.parse().expect("Invalid address");

    let mut stream = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    // Send a GET request to the root path
    stream.write_all(b"GET / HTTP/1.1\r\n\r\n")?;

    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;

    Ok(String::from_utf8_lossy(&buffer[..bytes_read]).to_string())
}


pub fn udp_banner(ip: &str, port: u32) -> io::Result<String> {
    let address = format!("{}:{}", ip, port);
    let socket_addr: SocketAddr = address.parse().expect("Invalid address");

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Some(Duration::new(5, 0)))?;
    socket.set_write_timeout(Some(Duration::new(5, 0)))?;
    socket.connect(socket_addr)?;
    let mut buffer = [0; 1024];
    
    match socket.recv(&mut buffer) {
        Ok(bytes_read) => {
            Ok(String::from_utf8_lossy(&buffer[..bytes_read]).to_string())
        }
        Err(e) => {
            Err(e)
        }
    }
}