use openai::AiResult;

pub mod openai;

pub async fn http_parse(response: reqwest::Response) -> String {
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

    println!("parsed: {:?}", parsed);

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
