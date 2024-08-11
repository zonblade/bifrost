use std::fs::File;
use std::io::Read;

use mini_config::Configure;

#[derive(Debug, Clone, Configure)]
pub enum MemData {
    PortData,
}

pub fn init(){
    // load data from file ./assets/csvjson.json
    let file_path = "./assets/csvjson.json";
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

    println!("Data: {}", data);
}