use crate::{config, http::http_parse};

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
        };

        println!("req: {:?}", req);

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

        http_parse(response).await
    }
}

fn query() -> String {
    r#"
list of common tcp, udp, database, file, server port. 
retrive at least 2 port each category (use real protocol analogy). 
list only, json list formatted {protocol, port, description}
exclude basic http [80,443]

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
{}
    "#,
    "list only, json list formatted {protocol, port, description}", 
    res,
    "note for result: minified array json plain text without formatting,make sure array closing are correct"
)
}
