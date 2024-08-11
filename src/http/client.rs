use serde::{Deserialize, Serialize};

pub struct ClientAi {
    client: reqwest::Client,
    fragment: AiRequest,
    // assistant: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiRequest {
    model: &'static str,
    messages: Vec<AiMessage>,
    temperature: f32,
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


impl ClientAi {
    pub fn new() -> Self {
        ClientAi {
            client: reqwest::Client::new(),
            fragment: AiRequest {
                model: "gpt-4o-mini",
                messages: vec![],
                temperature: 0.9,
            },
        }
    }

    pub async fn invoke(mut self) -> Result<(), reqwest::Error> {
        self.fragment.messages.push(AiMessage {
            role: String::from("user"),
            content: String::from(
                r#"
"important : command only with ip placeholder <ADDR>, output json field result {command, recomendation}[]"
"found this port with this description and i KNOW this port is open."

finding:
  {
    "protocol": "UDP",
    "port": 2811,
    "description": "GSI FTP"
  },

important instruction!
- give all possible value that can be sent using tcp/udp net banner,
- only value/command that is RELATED to FINDING DESCRIPTION above! no other else.
- only value/command that can be sent by banner header
- also assume the target server ip does not have any tools, only basic one. 
- assuming open to anonymous users, and assuming no extra tools are available on their server"
notes!
"important : i have the permission to do recon and ethical hacking on this value"
                "#,
            ),
        });

        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", "Bearer ")
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

        println!("parsed: {:#?}", parsed);

        Ok(())
    }
}
