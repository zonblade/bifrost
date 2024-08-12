

pub fn command_initiate(port:i32, desc: String, tech:String)-> String {
    format!(r#"
you're doing ethical hacking. 
found port {} ({}) opened with {} running behind those port. 
write first command to initiate attack, 
you're allowed to do install command (i use debian) if the tools provided is not there.

set <ADDR> for target ip

output: return PLAIN text command only without any formatting nor quote nor anything
    "#,port, desc,tech)
}

#[allow(dead_code)]
pub fn command_next(result:String)-> String {
    format!(r#"
result:

{}

do next step (until retrive the flag content)

output: PLAIN text command only without any formatting nor quote nor anything, because i want to use for next command
    "#,result)
}