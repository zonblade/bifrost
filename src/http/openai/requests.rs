use crate::{
    config,
    http::openai::{AiMessage, AiResult}, toolkit::prompter::assumption::command_suggest,
};

use super::{ClientAi, NextCommand};

impl ClientAi {
    pub async fn capabilities(mut self) -> Result<(), reqwest::Error> {
        let openkey = config::MemData::OpenKey.get_str();
        self.fragment.messages.push(AiMessage {
            role: String::from("user"),
            content: command_suggest(21, String::from("FTP"), String::from("Ubuntu-3ubuntu0.10")),
        });

        let response = self
            .client
            .post(config::OPENAI_URL)
            .header("Authorization", format!("Bearer {}", openkey))
            .json(&self.fragment)
            .send()
            .await?;

        let body = response.text().await?;

        let parsed = match serde_json::from_str::<AiResult>(&body) {
            Ok(parsed) => parsed,
            Err(e) => {
                println!("Error parsing response: {:?}", e);
                return Ok(());
            }
        };

        println!("Parsed: {:?}", parsed.choices);

        Ok(())
    }

    pub async fn invoke(mut self) -> Result<(), reqwest::Error> {
        let openkey = config::MemData::OpenKey.get_str();
        self.fragment.messages.push(AiMessage {
            role: String::from("user"),
            content: String::from(
                r#"
important : command only with ip <ADDR>, output json 
field {
    command, 
    recomendation, 
    canUseTcpUdpBannerToExecute
}[]

finding:
  - port: 21
    description: "File Transfer [Control]"
    protocol: "TCP"
    version: Ubuntu-3ubuntu0.10

important instruction!
- only value/command that is RELATED to FINDING DESCRIPTION above! no other else.
- also assume the target server ip does not have any tools, only basic one. 
- put default installed credential directly on the command for FTP Ubuntu-3ubuntu0.10
important notes!
- i have the permission to do recon and ethical hacking on this value
                "#,
            ),
        });

        let response = self
            .client
            .post(config::OPENAI_URL)
            .header("Authorization", format!("Bearer {}", openkey))
            .json(&self.fragment)
            .send()
            .await?;

        let body = response.text().await?;

        let parsed = match serde_json::from_str::<AiResult>(&body) {
            Ok(parsed) => parsed,
            Err(e) => {
                println!("Error parsing response: {:?}", e);
                return Ok(());
            }
        };

        let choices = parsed.choices;
        // if choices more than 1, get index 0
        let choice = match choices.len() {
            0 => {
                println!("No choices found");
                return Ok(());
            }
            _ => &choices[0],
        };

        let content = &choice
            .message
            .content
            .replace("```json", "")
            .replace("```", "");

        println!("Content: {}", content);

        let parsed_content = match serde_json::from_str::<Vec<NextCommand>>(content) {
            Ok(parsed_content) => parsed_content,
            Err(e) => {
                println!("Error parsing content: {:?}", e);
                return Ok(());
            }
        };

        for next_command in parsed_content {
            println!("Command: {}", next_command.command);
            println!("Recomendation: {}\n", next_command.recommendation);
        }

        Ok(())
    }

    pub async fn find_default_username_password(mut self) -> Result<(), reqwest::Error> {
        let openkey = config::MemData::OpenKey.get_str();
        self.fragment.messages.push(AiMessage {
            role: String::from("user"),
            content: String::from(
                r#"
                "#,
            ),
        });

        let response = self
            .client
            .post(config::OPENAI_URL)
            .header("Authorization", format!("Bearer {}", openkey))
            .json(&self.fragment)
            .send()
            .await?;

        let body = response.text().await?;

        let parsed = match serde_json::from_str::<AiResult>(&body) {
            Ok(parsed) => parsed,
            Err(e) => {
                println!("Error parsing response: {:?}", e);
                return Ok(());
            }
        };

        let choices = parsed.choices;

        let choice = match choices.len() {
            0 => {
                println!("No choices found");
                return Ok(());
            }
            _ => &choices[0],
        };

        let content = &choice
            .message
            .content
            .replace("```json", "")
            .replace("```", "");

        println!("Content: {}", content);

        Ok(())
    }
}
