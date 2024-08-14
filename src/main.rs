mod config;
mod http;
mod log;
mod toolkit;

use config::PortCSV;
use http::openai::ClientAi;
use log::printlg;
use toolkit::portscan::sweeper::PortScanner;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let ip = match args.get(1) {
        Some(ip) => ip.clone(),
        None => {
            printlg("Please provide an IP address".to_string());
            return;
        }
    };
    config::init().await;

    printlg("start scanning for open ports".to_string());
    let mut first_load = true;
    let mut result: Vec<PortScanner> = vec![];
    let mut feedback: String = String::new();
    loop {
        if !first_load {
            let known_ports = config::MemData::PortDataOld.get_str();
            let mut parsed_ports = match serde_json::from_str::<Vec<u32>>(&known_ports) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Failed to parse result: {:?}", e);
                    return;
                }
            };

            printlg("retriving new port list...".to_string());
            let format_feedback = format!("port before: {}, with feedback: {}",known_ports, &feedback);
            let new_port = ClientAi::new()
                .port_suggestion_re_opt(String::from(&format_feedback))
                .await;

            let res = match serde_json::from_str::<Vec<PortCSV>>(&new_port) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Failed to parse result: {:?}", e);
                    return;
                }
            };
            
            let port_res_only = res.iter().map(|port| port.port).collect::<Vec<u32>>();
            // merge to parsed_ports
            parsed_ports.extend(port_res_only);
            let parsed_ports = serde_json::to_string(&parsed_ports).unwrap();

            printlg("retriving done, scanning again...".to_string());
            let new_data = serde_json::to_string(&res).unwrap();
            config::MemData::PortData.set(&new_data);
            config::MemData::PortDataOld.set(&parsed_ports);
        }

        first_load = false;
        result = toolkit::portscan::sweeper::scan_port_assumption(ip.clone()).await;

        feedback = format!("found: {} ports open", &result.len());
        printlg(String::from(&feedback));

        if result.is_empty() {
            printlg("no open port found, aborting".to_string());
            continue;
        }
        printlg("initiating sequential attack on opened port\n".to_string());

        let mut cai = ClientAi::new();

        for port in result.clone() {
            let banner = toolkit::portscan::banner::grab_banner(&ip, port.port);
            let banner = match banner {
                Ok(banner) => banner,
                Err(e) => {
                    eprintln!("Error grabbing banner: {:?}", e);
                    continue;
                }
            };

            let parsed_banner = cai.banner_parse(banner).await;

            let desc = match port.desc {
                Some(desc) => desc,
                None => String::from("-"),
            };

            printlg(format!(
                "initiating attack \n\t-to port : {}\n\t-tech : {}\n\t-server : {}",
                port.port, desc, parsed_banner
            ));

            let res = cai.invoke(port.port as i32, desc, parsed_banner).await;
            let res = match res {
                Ok(res) => res.replace("<ADDR>", &ip),
                Err(e) => {
                    eprintln!("Error invoking AI: {:?}", e);
                    return;
                }
            };

            printlg(format!("res: {:?}", res));
        }

        break;
    }
}
