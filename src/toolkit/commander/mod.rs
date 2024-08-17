use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::io::ErrorKind;

use crate::http;
use crate::log::printlg;

use super::portscan::sweeper::PortScanner;

pub async fn terminal_session(ip: String, data: PortScanner) -> Result<(), i32> {
    let mut client = http::openai::ClientAi::new();

    let mut command = match client
        .invoke(data.port as i32, data.desc.unwrap(), data.head.unwrap())
        .await
    {
        Ok(command) => command,
        Err(_) => return Err(1),
    };

    // command = command.replace("<ADDR>", &ip);
    let command = String::from("ll");
    printlg(
        format!("Trying command: {}", command),
        crossterm::style::Color::White,
    );

    loop {
        // Print the command to debug
        printlg(
            format!("Executing command: {}", command),
            crossterm::style::Color::Yellow,
        );

        // Split the command into the executable and its arguments
        let mut parts = command.split_whitespace();
        let executable = parts.next().unwrap();
        let args: Vec<&str> = parts.collect();

        // Spawn the command and capture stdout and stderr
        let output = match Command::new(executable)
            .args(&args)
            .stdout(Stdio::piped())  // Capture stdout
            .stderr(Stdio::piped())  // Capture stderr
            .output()
        {
            Ok(output) => output,
            Err(e) => {
                // Check if the error is because the command wasn't found
                if e.kind() == ErrorKind::NotFound {
                    eprintln!("Command not found: {}", executable);
                } else {
                    eprintln!("Failed to execute command: {}", e);
                }
                return Err(1);
            }
        };

        // Print the command's output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !stdout.is_empty() {
            println!("Command output:\n{}", stdout);
        }
        if !stderr.is_empty() {
            eprintln!("Command error output:\n{}", stderr);
        }

        break;
    }

    Ok(())
}
