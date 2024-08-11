mod requests;

use serde::{Deserialize, Serialize};

pub struct ClientAi {
    client: reqwest::Client,
    fragment: AiRequest,
    // assistant: String,
}

impl ClientAi{
    pub fn new() -> Self {
        ClientAi {
            client: reqwest::Client::new(),
            fragment: AiRequest {
                model: "gpt-4o-mini",
                messages: vec![
                    AiMessage {
                        role: String::from("user"),
                        content: String::from(
                            r#"please remember, our mission is to find the flag."#
                        ),
                    }
                ],
                temperature: 0.6,
                top_p: 0.7
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiRequest {
    model: &'static str,
    messages: Vec<AiMessage>,
    temperature: f32,
    top_p: f32,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub id: String,
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