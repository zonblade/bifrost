
use crossterm::style::Color;

use crate::{config, log::printlg, toolkit::prompter::assumption::command_next};

use super::{AiMessage, AiResult, ClientAi};

impl ClientAi {
    pub async fn intruder(&mut self, message: String) -> String {
        let openkey = config::MemData::OpenKey.get_str();

        let formatted_message = command_next(message);

        self.fragment.messages.push(AiMessage {
            role: String::from("user"),
            content: formatted_message,
        });

        let response = match self
            .client
            .post(config::OPENAI_URL)
            .header("Authorization", format!("Bearer {}", openkey))
            .json(&self.fragment)
            .send()
            .await
        {
            Ok(response) => response,
            Err(e) => {
                printlg(format!("Error sending request: {:?}", e), Color::Red);
                return String::from("-");
            }
        };

        let body = match response.text().await {
            Ok(body) => body,
            Err(e) => {
                printlg(format!("Error parsing response: {:?}", e), Color::Red);
                return String::from("-");
            }
        };

        let parsed = match serde_json::from_str::<AiResult>(&body) {
            Ok(parsed) => parsed,
            Err(e) => {
                printlg(format!("Error parsing response: {:?}", e), Color::Red);
                return String::from("-");
            }
        };

        let choices = parsed.choices;
        // if choices more than 1, get index 0
        let choice = match choices.len() {
            0 => {
                printlg(format!("No choices found"), Color::Red);
                return String::from("-");
            }
            _ => &choices[0],
        };

        self.fragment.messages.push(AiMessage {
            role: String::from("assistant"),
            content: choice.message.content.clone(),
        });

        choice.message.content.clone()
    }
}