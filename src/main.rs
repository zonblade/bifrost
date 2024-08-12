mod config;
mod http;
mod log;
mod toolkit;

use http::openai::ClientAi;
use log::printlg;

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

    let result = toolkit::portscan::sweeper::scan_port_assumption(ip.clone()).await;

    printlg(format!("found: {} ports open", result.len()));

    if result.is_empty() {
        printlg("no open port found, aborting".to_string());
        return;
    }
    printlg("initiating sequential attack on opened port\n".to_string());

    let mut cai = ClientAi::new();

    for port in result {
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
}
