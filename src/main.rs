mod config;
mod http;
mod log;
mod toolkit;

use config::PortCSV;
use crossterm::style::Color;
use http::openai::ClientAi;
use log::printlg;
use toolkit::{commander::terminal_session, portscan::sweeper::PortScanner};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let ip = match args.get(1) {
        Some(ip) => ip.clone(),
        None => {
            printlg("Please provide an IP address".to_string(), Color::Red);
            return;
        }
    };

    config::init().await;
    printlg("start scanning for open ports".to_string(), Color::White);
    let mut first_load = true;
    let mut feedback: String = String::new();
    let mut client_port = ClientAi::new();
    let mut fingerprint: Vec<PortScanner> = vec![];
    let mut cai_port = ClientAi::new();
    let mut result: Vec<PortScanner> = vec![];
    let port_scan_limit = 10;
    let mut port_scan_count = 0;
    let finger_print_limit = 10;
    let mut finger_print_count = 0;

    loop {
        if finger_print_count >= finger_print_limit {
            printlg(
                "fingerprint limit reached, continue with just as it is.".to_string(),
                Color::Magenta,
            );
            break;
        }

        loop {
            if port_scan_count >= port_scan_limit {
                printlg(
                    "port scan limit reached, continue with just as it is.".to_string(),
                    Color::Magenta,
                );
                break;
            }
            port_scan_count += 1;
            if !first_load {
                toolkit::portscan::port_loader::loader(&mut client_port, feedback.clone()).await;
            }

            first_load = false;

            result = toolkit::portscan::sweeper::scan_port_assumption(ip.clone()).await;
            feedback = format!("found: {} ports open", &result.len());

            if result.is_empty() {
                printlg("no open port found".to_string(), Color::Red);
                continue;
            }
            break;
        }

        for port in result.clone() {
            if port.proto.is_none() {
                continue;
            }

            let port = PortCSV {
                port: port.port,
                description: port.desc.unwrap(),
                protocol: port.proto.unwrap(),
                version: None,
            };

            let mut banner = String::new();
            match port.protocol.as_str() {
                protocol if protocol == "TCP" => {
                    let res = toolkit::portscan::banner::tcp_banner(&ip, port.port);
                    banner = match res {
                        Ok(banner) => banner,
                        Err(e) => {
                            printlg(
                                format!(
                                    "{}:{:?} |TCP| Error grabbing banner: {:?}",
                                    &ip, port.port, e
                                ),
                                Color::Red,
                            );
                            continue;
                        }
                    };
                }
                protocol if protocol == "UDP" => {
                    let res = toolkit::portscan::banner::udp_banner(&ip, port.port);
                    banner = match res {
                        Ok(banner) => banner,
                        Err(e) => {
                            printlg(
                                format!(
                                    "{}:{:?} |UDP| Error grabbing banner: {:?}",
                                    &ip, port.port, e
                                ),
                                Color::Red,
                            );
                            continue;
                        }
                    };
                }
                _ => {
                    printlg("Unknown protocol".to_string(), Color::Red);
                    continue;
                }
            }

            let parsed_banner = cai_port.banner_parse(banner).await;

            let new_data = PortScanner {
                port: port.port,
                desc: Some(port.description),
                proto: Some(port.protocol),
                open: true,
                head: Some(parsed_banner),
            };

            fingerprint.push(new_data);
        }

        if fingerprint.is_empty() {
            finger_print_count += 1;
            printlg("no fingerprint found, retry...".to_string(), Color::Red);
            continue;
        }

        break;
    }
    println!("");

    printlg(format!("fingerprint: {:#?}", fingerprint), Color::Cyan);

    for port in fingerprint {
        let _ = terminal_session(ip.clone(), port).await;
    }

    printlg("done".to_string(), Color::Green);
}
