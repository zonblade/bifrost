use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct PortScan {
    pub protocol: String,
    pub port: u16,
    pub description: String,
}