use crossterm::style::Color;

use crate::{
    config, http::openai::{AiMessage, AiResult}, log::printlg, toolkit::prompter::assumption::command_initiate
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
        self.fragment.messages.push(AiMessage {
            role: String::from("user"),
            content: String::from(r#"
if any username/password need to be provided, 
please assume to use anonymous or default username/password first.
then you can assume. if the flag is found download the flag and stop the intrusion, 
type command "end" to stop the intrusion program.
            "#),
        });
        self.fragment.messages.push(AiMessage {
            role: String::from("user"),
            content: String::from(r#"
after installing the command or software, recomended to do a quick check
on help menu or version command to see if the command is installed correctly.
and grab the available command to use.
            "#),
        });
        self.fragment.messages.push(AiMessage {
            role: String::from("user"),
            content: String::from(r#"
make sure if any address or command did not accidently put space
FOR EXAMPLE:
http: //domain.com < this should be http://domain.com
also npm i-D this should be npm i -D.
            "#),
        });
        self.fragment.messages.push(AiMessage {
            role: String::from("user"),
            content: String::from(r#"
if the command need username password you may try
- anonymous/anonymous
- admin/admin
- root/root
- user/user
- guest/guest
or replace password with P@ssw0rd or default
or try to put empty password.
            "#),
        });

        self.fragment.model = config::MemData::OpenModel.get_str();
        self.fragment.temperature= 0.97;
        self.fragment.top_p= 1.0;
        self.fragment.frequency_penalty= 2.0;
        self.fragment.presence_penalty= 2.0;
        self.fragment.max_tokens= 10_000;

        let response = match self
            .client
            .post(config::OPENAI_URL)
            .header("Authorization", format!("Bearer {}", openkey))
            .json(&self.fragment)
            .send()
            .await {
            Ok(response) => response,
            Err(e) => {
                printlg(format!("Error sending request: {:?}", e), Color::Red);
                return Err(1);
            }
        };

        if !response.status().is_success() {
            printlg(format!("Error response: {:?}", response), Color::Red);
            return Err(1);
        }

        let body = match response.text().await {
            Ok(body) => body,
            Err(e) => {
                printlg(format!("Error parsing response: {:?}", e), Color::Red);
                return Err(1);
            }
        };

        let parsed = match serde_json::from_str::<AiResult>(&body) {
            Ok(parsed) => parsed,
            Err(e) => {
                printlg(format!("Error parsing response: {:?}", e), Color::Red);
                return Err(2);
            }
        };

        let choices = parsed.choices;
        // if choices more than 1, get index 0
        let choice = match choices.len() {
            0 => {
                printlg(format!("No choices found"), Color::Red);
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
