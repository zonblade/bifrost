use std::fs::File;
use std::io::Read;

use mini_config::Configure;
use serde::{Deserialize, Serialize};

pub(crate) const OPENAI_URL: &str = "https://api.openai.com/v1/chat/completions";

#[derive(Debug, Clone, Configure)]
pub enum MemData {
    PortData,
    OpenKey
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortCSV {
    pub port: u16,
    pub description: String,
    pub protocol: String,
    pub version: Option<String>,
}

pub fn init(){
    dotenv::dotenv().ok();
    
    let file_path = "./assets/assumption.json";
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
    MemData::PortData.set(&data);

    let openkey = std::env::var("OPENKEY").expect("should have OPENKEY");
    MemData::OpenKey.set(&openkey);
}