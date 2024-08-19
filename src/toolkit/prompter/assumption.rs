

pub fn command_initiate(port:i32, desc: String, tech:String)-> String {
    format!(r#"
you're doing ethical hacking. 
use nmap for first recon.
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

do next step (
- until retrive the flag content, 
- flag may be have different name be smart dude!
- just try to see the content if got anything suspicious
)
if objective achieved do write "end" as command!
USE NON INTERACTIVE COMMAND, PREFFERABLY INLINE USING EOF OR SOMETHING INLINE!
IF THE COMMAND NOT EXIST, INSTALL THE COMMAND FIRST, THEN RETRY!
IF FEW TIMES RETRY STILL NOT WORK, FIND OTHER ALTERNATIVE!!

USABLE OUTPUT PLAIN text command only without any formatting nor quote nor anything

sidenotes:
make sure if any address or command did not accidently put space
FOR EXAMPLE:
http: // domain.com < this should be http://domain.com
also npm i-D this should be npm i -D.
    "#,result)
}