use std::fs::File;
use std::io::Read;

use mini_config::Configure;
use serde::{Deserialize, Serialize};

use crate::{http::openai::ClientAi, log::printlg};

// use crate::log::printlg;

pub(crate) const OPENAI_URL: &str = "https://api.openai.com/v1/chat/completions";

#[derive(Debug, Clone, Configure)]
pub enum MemData {
    PortData,
    OpenKey,
    OpenModel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortCSV {
    pub port: u16,
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
            println!("nmap");
            // execute command if there is any nmap installed
            let output = std::process::Command::new("nmap").arg("-v").output();
            match output {
                Ok(output) => {
                    println!("output: {:?}", output);
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
                    println!("res: {:?}", res);
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


    let file_path = "./assets/basic.json";
    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open file: {}", e);
            return;
        }
    };

    let mut data = String::new();
    if let Err(e) = file.read_to_string(&mut data) {
        eprintln!("Failed to read file: {}", e);
        return;
    }

    if use_generated_port {
        printlg("using generated port, generating...".to_string());
        let result = ClientAi::new().port_suggestion().await;
        let result = result.replace("\n", "");
        println!("result: {:?}", result);

        let res = match serde_json::from_str::<Vec<PortCSV>>(&result) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Failed to parse result: {:?}", e);
                return;
            }
        };
        
        data = serde_json::to_string(&res).unwrap();
    }

    MemData::PortData.set(&data);
}
