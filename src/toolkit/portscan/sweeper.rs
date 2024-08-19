use std::net::{SocketAddr, TcpStream};
use std::time::Duration;
use crossterm::style::Color;
use tokio::sync::mpsc;
use tokio::task;

use crate::log::{printlg, printlsc};
use crate::{config, toolkit};

#[derive(Debug, Clone)]
pub struct PortScanner {
    pub port: u32,
    #[allow(dead_code)]
    pub open: bool,
    pub proto: Option<String>,
    pub desc: Option<String>,
    #[allow(dead_code)]
    pub head: Option<String>,
}

pub async fn scan_ports(ip: &str, ports: &[u32]) -> Vec<PortScanner> {
    let mut open_ports = vec![];
    for &port in ports {
        printlsc(format!("Scanning port {}\r", port));
        let address = format!("{}:{}", ip, port);
        let socket_addr: SocketAddr = address.parse().expect("Invalid address");
        match TcpStream::connect_timeout(&socket_addr, Duration::from_millis(2000)) {
            Ok(_) => {
                open_ports.push(PortScanner {
                    port,
                    open: true,
                    desc: None,
                    head: None,
                    proto: None,
                });
            }
            Err(_) => continue,
        }
    }
    open_ports
}

pub async fn scan_port_assumption(ip: String) -> Vec<PortScanner> {
    let known_ports = config::MemData::PortData.get_str();
    let parsed_known_ports = match serde_json::from_str::<Vec<config::PortCSV>>(&known_ports) {
        Ok(parsed_known_ports) => parsed_known_ports,
        Err(e) => {
            printlg(format!("Error parsing known ports: {:?}", e), Color::Red);
            return vec![];
        }
    };
    let parsed_ports = parsed_known_ports.clone();

    let (tx, mut rx) = mpsc::channel(1000); // Increase buffer size

    tokio::spawn(async move {
        let mut handles = vec![];
        for chunk in parsed_known_ports.chunks(50) {
            // Process ports in batches of 10
            let ip = ip.clone();
            let tx = tx.clone();
            let ports: Vec<u32> = chunk.iter().map(|port| port.port).collect();

            // let first_port = ports.first().unwrap();
            // let last_port = ports.last().unwrap();

            let handle = task::spawn(async move {
                let result = toolkit::portscan::sweeper::scan_ports(&ip, &ports).await;
                tx.send(result).await.unwrap();
            });
            handles.push(handle);
        }

        for handle in handles {
            if let Err(e) = handle.await {
                printlg(format!("Error scanning ports: {:?}", e), Color::Red);
            }
        }
    });

    let mut results = vec![];

    while let Some(result) = rx.recv().await {
        results.push(result);
    }

    let mut reduce_result = vec![];

    for result in results {
        reduce_result.extend(result);
    }

    for port in reduce_result.iter_mut() {
        for known_port in parsed_ports.iter() {
            if port.port == known_port.port {
                port.desc = Some(known_port.description.clone());
                port.proto = Some(known_port.protocol.clone());
            }
        }
    }

    reduce_result
}
