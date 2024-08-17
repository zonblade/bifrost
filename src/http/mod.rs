use crossterm::style::Color;
use openai::AiResult;
use regex::Regex;

use crate::log::printlg;

pub mod openai;

pub async fn http_parse(response: reqwest::Response) -> String {
    let mut body = match response.text().await {
        Ok(body) => body,
        Err(e) => {
            printlg(format!("Error parsing response: {:?}", e), Color::Red);
            return String::from("-");
        }
    };

    body = body.replace("```json","");
    body = body.replace("```","");
    // remove new line
    body = body.replace("\n", "");
    // remove space in front and back
    body = body.trim().to_string();

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
            return String::from("-");
        }
        _ => &choices[0],
    };

    String::from(choice.message.content.clone())
}
