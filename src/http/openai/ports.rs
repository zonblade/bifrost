use crate::config;

use super::{AiMessage, AiRequest, AiResult, ClientAi};

impl ClientAi {
    pub async fn port_suggestion(&mut self) -> String {
        let openkey = config::MemData::OpenKey.get_str();
        let model = config::MemData::OpenModel.get_str();

        let formatted_message = query();

        let req = AiRequest {
            messages: vec![AiMessage {
                content: formatted_message,
                role: String::from("user"),
            }],
            model,
            temperature: 0.49,
            top_p: 0.13,
            frequency_penalty: 0.41,
            presence_penalty: 0.66,
        };

        let response = match self
            .client
            .post(config::OPENAI_URL)
            .header("Authorization", format!("Bearer {}", openkey))
            .json(&req)
            .send()
            .await
        {
            Ok(response) => response,
            Err(e) => {
                println!("Error sending request: {:?}", e);
                return String::from("-");
            }
        };

        let body = match response.text().await {
            Ok(body) => body,
            Err(e) => {
                println!("Error reading response: {:?}", e);
                return String::from("-");
            }
        };

        let parsed = match serde_json::from_str::<AiResult>(&body) {
            Ok(parsed) => parsed,
            Err(e) => {
                println!("Error parsing response: {:?}", e);
                return String::from("-");
            }
        };

        let choices = parsed.choices;
        // if choices more than 1, get index 0
        let choice = match choices.len() {
            0 => {
                println!("No choices found");
                return String::from("-");
            }
            _ => &choices[0],
        };

        String::from(choice.message.content.clone())
    }
}

fn query() -> String {
    r#"
list of common http, tcp, udp, database, file, server port. 
retrive at least 2 port each category (use real protocol analogy). 
list only, json list formatted {protocol, port, description}

note for result: minified array json plain text without formatting
make sure array closing are correct
"#
    .to_string()
}
