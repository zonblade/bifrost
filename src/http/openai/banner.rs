use crossterm::style::Color;

use crate::{config, log::printlg};

use super::{AiMessage, AiRequest, AiResult, ClientAi};

impl ClientAi {
    pub async fn banner_parse(&mut self, banner: String) -> String {
        let openkey = config::MemData::OpenKey.get_str();
        let model = config::MemData::OpenModel.get_str();

        if banner.len() < 1 {
            return String::from("-");
        }
        let formatted_message = query(banner);

        let req = AiRequest {
            messages: vec![AiMessage {
                content: formatted_message,
                role: String::from("user"),
            }],
            model,
            temperature: 0.9,
            top_p: 1.0,
            frequency_penalty: 0.61,
            presence_penalty: 1.16,
            max_tokens: 1_000,
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

        String::from(choice.message.content.clone())
    }
}

fn query(banner: String) -> String {
    format!(
        r#"
result :
{}
write detected technology/os/tools based on result above,
if detected any version/detail please include it to the list.
return list only separated by comma
"#,
        banner
    )
}
