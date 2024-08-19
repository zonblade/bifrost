use crossterm::style::Color;
use serde::{Deserialize, Serialize};

use crate::{
    config::{self, PortCSV},
    http::openai::ClientAi,
    log::printlg,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct PortScan {
    pub protocol: String,
    pub port: u32,
    pub description: String,
}

pub async fn loader(client: &mut ClientAi, feedback: String) {
    let known_ports = config::MemData::PortDataOld.get_str();
    let mut parsed_ports = match serde_json::from_str::<Vec<u32>>(known_ports) {
        Ok(result) => result,
        Err(e) => {
            printlg(format!("Failed to parse result: {:?}", e), Color::Red);
            return;
        }
    };

    printlg("retriving new port list...".to_string(), Color::Cyan);
    let format_feedback = format!("port before: {}, with feedback: {}", known_ports, &feedback);
    let new_port = client.port_suggestion_re_opt(String::from(&format_feedback))
        .await;

    let res = match serde_json::from_str::<Vec<PortCSV>>(&new_port) {
        Ok(result) => result,
        Err(e) => {
            printlg(format!("Failed to parse result: {:?}", e), Color::Red);
            return;
        }
    };
    
    let port_res_only = res.iter().map(|port| port.port).collect::<Vec<u32>>();

    printlg(format!("generated port: {:?}", port_res_only), Color::Cyan);
    // merge to parsed_ports
    parsed_ports.extend(port_res_only);
    let parsed_ports = serde_json::to_string(&parsed_ports).unwrap();
    
    printlg("retriving done, scanning again...".to_string(), Color::DarkGreen);
    let new_data = serde_json::to_string(&res).unwrap();
    config::MemData::PortData.set(&new_data);
    config::MemData::PortDataOld.set(&parsed_ports);
}
