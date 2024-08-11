pub struct ClientHttp {
    client: reqwest::Client,
}

impl ClientHttp {
    pub fn new() -> Self {
        ClientHttp {
            client: reqwest::Client::new()
        }
    }
}
