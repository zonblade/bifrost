mod config;
mod http;
mod log;
mod toolkit;

use std::sync::Arc;

use config::PortCSV;
use crossterm::style::Color;
use futures::future::join_all;
use http::openai::ClientAi;
use log::printlg;
use tokio::task;
use tokio::sync::Mutex;
use toolkit::{commands::loop_main::terminal_thread, portscan::sweeper::PortScanner};

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

    // test ping to google.com to determine if connection is ok
    // let _ = std::process::Command::new("ping")
    //     .arg("-c")
    //     .arg("1")
    //     .arg("google.com")
    //     .output()
    //     .expect("failed to execute process, aborting");

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
        println!("");
        printlg("scanning for fingerprint, may take a while...".to_string(), Color::Green);

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
                "TCP" => {
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
                "UDP" => {
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
                    printlg("Unknown protocol, might be possible to explore".to_string(), Color::Red);
                }
            }

            let mut parsed_banner = cai_port.banner_parse(banner).await;

            // split max 200 char
            if parsed_banner.len() > 200 {
                parsed_banner = parsed_banner[..200].to_string();
            }

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

    if !fingerprint.is_empty() {
        printlg("fingerprint found...".to_string(), Color::Green);
        printlg("running attack session...".to_string(), Color::Green);
    }

    run_sessions(ip, fingerprint).await;

    printlg("done".to_string(), Color::Green);
}

async fn run_sessions(ip: String, fingerprint: Vec<PortScanner>) {
    // let mut tasks = Vec::new();
    let previous_output = Arc::new(Mutex::new(String::new()));

    for (index, port) in fingerprint.into_iter().enumerate() {
        let ip_clone = ip.clone();
        let previous_output_clone = Arc::clone(&previous_output);
        let _ = terminal_thread(index as i32, ip_clone, port, previous_output_clone).await;
        // let task = task::spawn(async move {
        // });
        // tasks.push(task);
    }

    // Wait for all tasks to complete
    // let _ = join_all(tasks).await;
}
