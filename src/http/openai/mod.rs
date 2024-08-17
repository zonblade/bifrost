mod invoke;
mod banner;
mod intruder;
pub mod ports;

use serde::{Deserialize, Serialize};

use crate::config;

#[derive(Debug, Clone)]
pub struct ClientAi {
    client: reqwest::Client,
    fragment: AiRequest,
    // assistant: String,
}

impl ClientAi{
    pub fn new() -> Self {
        let model = config::MemData::OpenModel.get_str();
        ClientAi {
            client: reqwest::Client::new(),
            fragment: AiRequest {
                model,
                messages: vec![
                    AiMessage {
                        role: String::from("user"),
                        content: String::from(
                            r#"
[below this chat is start over, does not matter chat before this]
remember, you're ethical hacker.

note about tools:
- use tools like nmap, sqlmap, curl, telnet, etc (basic tools)
- use linux os
- you are allowed to run install command for the tools if it's not installed
- do not use tools that require to input file!
"#
                        ),
                    }
                ],
                temperature: 0.641123421,
                top_p: 0.698272817,
                frequency_penalty: 0.00,
                presence_penalty: 0.00,
                max_tokens: 10238,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRequest {
    model: &'static str,
    messages: Vec<AiMessage>,
    temperature: f32,
    top_p: f32,
    presence_penalty: f32,
    frequency_penalty: f32,
    max_tokens: i32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiAssistant {
    pub instructions: String,
    pub name: String,
    pub tools: Vec<AiAsistantType>,
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiAsistantType {
    #[serde(rename = "type")]
    pub tipe: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiResult {
    pub id: Option<String>,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
    pub system_fingerprint: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Choice {
    pub index: i64,
    pub message: Message,
    pub finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub refusal: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct NextCommand {
    pub command: String,
    pub recommendation: String,
}