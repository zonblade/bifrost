

pub fn command_suggest(port:i32, desc: String, tech:String)-> String {
    format!(r#"
    for port {} (known as {}) those port run on {}, 
    write a step by step command to interact with this port.
    but we only able to send package to this port using rust TcpStream,
    we cannot run cli command in our machine nor using any tools outside TcpStream.
    "#,port, desc,tech)
}