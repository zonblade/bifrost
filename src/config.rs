use std::fs::File;
use std::io::Read;

use crossterm::style::Color;
use mini_config::Configure;
use serde::{Deserialize, Serialize};

use crate::{http::openai::ClientAi, log::printlg};

// use crate::log::printlg;

pub(crate) const OPENAI_URL: &str = "https://api.openai.com/v1/chat/completions";

#[derive(Debug, Clone, Configure)]
pub enum MemData {
    PortData,
    PortDataOld,
    OpenKey,
    OpenModel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortCSV {
    pub port: u32,
    pub description: String,
    pub protocol: String,
    pub version: Option<String>,
}

pub async fn init() {
    dotenv::dotenv().ok();
    // check if this can run sudo
    // let _ = std::process::Command::new("sudo")
    //     .arg("ls")
    //     .output()
    //     .expect("[FAL] failed to execute process, aborting");

    // get args --nmap
    let args: Vec<String> = std::env::args().collect();
    let mut use_generated_port = false;
    match (args.len() > 1, args.get(2).map(|s| s.as_str())) {
        (true, Some("assume-port")) => {
            use_generated_port = true;
        }
        (true, Some("nmap")) => {
            printlg("checking nmap...".to_string(), Color::White);
            // execute command if there is any nmap installed
            let output = std::process::Command::new("nmap").arg("-v").output();
            match output {
                Ok(_) => {
                    printlg("nmap is installed".to_string(), Color::Green);
                    std::process::exit(0);
                }
                Err(_) => {
                    let res = std::process::Command::new("sudo")
                        .arg("apt-get")
                        .arg("install")
                        .arg("nmap")
                        .arg("-y")
                        .output()
                        .expect("[FAL] failed to execute process, aborting");
                    printlg(format!("nmap installed: {:?}", res), Color::Green);
                }
            }
        }
        _ => {}
    }

    let openkey = std::env::var("OPEN_KEY").expect("should have OPENKEY");
    MemData::OpenKey.set(&openkey);

    let openmodel = std::env::var("OPEN_MODEL").expect("should have OPEN_MODEL");
    MemData::OpenModel.set(&openmodel);

    // initiate model to get common ports

    let file_path = "./assets/mini.json";
    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => {
            printlg(format!("Failed to open file: {}", e), Color::Red);
            return;
        }
    };

    let mut data = String::new();
    if let Err(e) = file.read_to_string(&mut data) {
        printlg(format!("Failed to read file: {}", e), Color::Red);
        return;
    }

    if use_generated_port {
        printlg(
            "using generated port, generating...".to_string(),
            Color::White,
        );
        let result = ClientAi::new().port_suggestion().await;
        let result = result.replace("\n", "");

        let res = match serde_json::from_str::<Vec<PortCSV>>(&result) {
            Ok(result) => result,
            Err(e) => {
                printlg(format!("Failed to parse result: {:?}", e), Color::Red);
                return;
            }
        };
        let port_only = res.iter().map(|port| port.port).collect::<Vec<u32>>();

        printlg("generating done, scanning...".to_string(), Color::White);
        printlg(format!("generated port: {:?}", port_only), Color::Cyan);

        data = serde_json::to_string(&res).unwrap();
    }

    MemData::PortData.set(&data);

    let parsed_data = match serde_json::from_str::<Vec<PortCSV>>(&data) {
        Ok(parsed_data) => parsed_data,
        Err(e) => {
            printlg(format!("Failed to parse result: {:?}", e), Color::Red);
            return;
        }
    };

    let parsed_port_only = parsed_data
        .iter()
        .map(|port| port.port)
        .collect::<Vec<u32>>();

    let parsed_port_only = serde_json::to_string(&parsed_port_only).unwrap();
    MemData::PortDataOld.set(&parsed_port_only);
}
