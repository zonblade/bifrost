use crossterm::style::Color;

use crate::{config, http::http_parse, log::printlg};

use super::{AiMessage, AiRequest, ClientAi};

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
            max_tokens: 10826,
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

        http_parse(response).await
    }

    pub async fn port_suggestion_re_opt(&mut self, res: String) -> String {
        let openkey = config::MemData::OpenKey.get_str();
        let model = config::MemData::OpenModel.get_str();

        let formatted_message = query_re_opt(res);

        let req = AiRequest {
            messages: vec![AiMessage {
                content: formatted_message,
                role: String::from("user"),
            }],
            model,
            temperature: 0.59,
            top_p: 0.13,
            frequency_penalty: 0.41,
            presence_penalty: 0.66,
            max_tokens: 10824,
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

        http_parse(response).await
    }
}

fn query() -> String {
    r#"
list of common tcp, udp, database, file, server port, etc.
retrive at least 2-10 port each category (use real protocol analogy). 
list only, json list formatted {protocol, port, description}
exclude basic http [80,443]
exclude DNS port [53]

note for result: minified array json plain text without formatting
make sure array closing are correct
"#
    .to_string()
}

fn query_re_opt(res: String) -> String {
    format!(r#"
{}

do not use port that already listed below:
{}

find other common port possibilities based on CVE, NIST, or other sources.
IMPORTANT TO REMOVE 80, 443, 53 FROM RESULT
{}
    "#,
    "list only, json list formatted {protocol, port, description}", 
    res,
    "note for result: minified array json plain text without formatting,make sure array closing are correct"
)
}
