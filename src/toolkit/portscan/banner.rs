use std::net::{TcpStream, SocketAddr};
use std::io::{self, Write, Read};
use std::time::Duration;

pub fn grab_banner(ip: &str, port: u16) -> io::Result<String> {
    let address = format!("{}:{}", ip, port);
    let socket_addr: SocketAddr = address.parse().expect("Invalid address");

    let mut stream = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    // Send a GET request to the root path
    stream.write_all(b"ssh hpp@IP\r\n\r\n")?;

    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;

    Ok(String::from_utf8_lossy(&buffer[..bytes_read]).to_string())
}