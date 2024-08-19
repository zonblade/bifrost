

pub fn command_initiate(port:i32, desc: String, tech:String)-> String {
    format!(r#"
you're doing ethical hacking. 
found port {} ({}) opened with {} running behind those port. 
write first command to initiate intrusion.
NOTE : if the result indicate script or command not found, 
install the command and MUST RETRY!

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
if objective achieved do write "end" as command!
IF THE COMMAND NOT EXIST, INSTALL THE COMMAND FIRST, THEN RETRY!

USABLE OUTPUT PLAIN text command only without any formatting nor quote nor anything
    "#,result)
}