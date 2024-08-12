use crate::config;

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
