use crate::{
    config,
    http::openai::{AiMessage, AiResult},
    toolkit::prompter::assumption::command_initiate,
};

use super::ClientAi;

impl ClientAi {
    pub async fn invoke(
        &mut self,
        port: i32,
        desc: String,
        tech: String,
    ) -> Result<String, i32> {
        let openkey = config::MemData::OpenKey.get_str();
        self.fragment.messages.push(AiMessage {
            role: String::from("user"),
            content: command_initiate(port, desc, tech),
        });

        let response = match self
            .client
            .post(config::OPENAI_URL)
            .header("Authorization", format!("Bearer {}", openkey))
            .json(&self.fragment)
            .send()
            .await {
            Ok(response) => response,
            Err(e) => {
                println!("Error sending request: {:?}", e);
                return Err(1);
            }
        };

        if !response.status().is_success() {
            println!("Error response: {:?}", response.status());
            return Err(1);
        }

        let body = match response.text().await {
            Ok(body) => body,
            Err(e) => {
                println!("Error reading response: {:?}", e);
                return Err(1);
            }
        };

        let parsed = match serde_json::from_str::<AiResult>(&body) {
            Ok(parsed) => parsed,
            Err(e) => {
                println!("Error parsing response: {:?}", e);
                return Err(2);
            }
        };

        let choices = parsed.choices;
        // if choices more than 1, get index 0
        let choice = match choices.len() {
            0 => {
                println!("No choices found");
                return Err(3);
            }
            _ => &choices[0],
        };

        self.fragment.messages.push(AiMessage {
            role: choice.message.role.clone(),
            content: choice.message.content.clone(),
        });

        Ok(choice.message.content.clone())
    }
}
