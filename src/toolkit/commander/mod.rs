mod child;

use std::io::{self, BufRead, BufReader, ErrorKind, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use crossterm::execute;
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};

use crate::http::openai::ClientAi;
use crate::log::printlg;

use super::portscan::sweeper::PortScanner;

pub async fn terminal_session(ip: String, data: PortScanner) -> Result<(), i32> {
    // 0 = init, 1 = feedback, 2 = next command, 3 = exit
    let command_lock = Arc::new((Mutex::new(0), Condvar::new()));
    let command_feedback = Arc::new(Mutex::new("".to_string()));
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let mut commander = String::new();
    let client = Arc::new(Mutex::new(ClientAi::new()));

    loop {
        let init_data = data.clone();
        let (lock, cvar) = &*command_lock;
        let state = {
            let lock = lock.lock().unwrap();
            *lock
        };

        match state {
            0 => {
                println!("State 0: Initializing command");
                let pew_command = client
                    .lock()
                    .unwrap()
                    .invoke(
                        init_data.port as i32,
                        init_data.desc.unwrap(),
                        init_data.head.unwrap(),
                    )
                    .await;
                println!("Command: {:?}", pew_command);
                let mut pew_command = match pew_command {
                    Ok(pew_command) => {
                        let mut lock = lock.lock().unwrap();
                        *lock = 2;
                        cvar.notify_all();
                        pew_command
                    }
                    Err(_) => {
                        printlg("Error invoking command".to_string(), Color::Red);
                        return Err(1);
                    }
                };
                pew_command = pew_command.trim().to_string();
                pew_command = pew_command.replace("<ADDR>", &ip);
                pew_command = pew_command.replace("sudo", "");
                commander = pew_command;
            }
            2 => {
                println!("State 2: Next command");
                let feedback = command_feedback.lock().unwrap();
                let mut pew_command = client
                    .lock()
                    .unwrap()
                    .intruder(String::from(feedback.clone()))
                    .await;
                print!("Command: {:?}", pew_command);
                let mut lock = lock.lock().unwrap();
                *lock = 4;
                cvar.notify_all();
                pew_command = pew_command.trim().to_string();
                pew_command = pew_command.replace("sudo", "");
                commander = pew_command;
                continue;
            }
            3 => {
                println!("State 3: Exiting");
                return Ok(());
            }
            _ => {}
        }

        let initial_command = commander.trim();
        println!("Initial command: {}", initial_command);

        if initial_command.eq_ignore_ascii_case("exit") {
            println!("Exit command received");
            break;
        }

        let mut parts = initial_command.split_whitespace();
        let executable = parts.next().unwrap();
        let args: Vec<&str> = parts.collect();

        println!("Executing command: {} with args: {:?}", executable, args);

        let mut child = match Command::new(executable)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => {
                let mut feedback = command_feedback.lock().unwrap();
                *feedback = match e.kind() {
                    ErrorKind::NotFound => format!("Command not found: {}", executable),
                    ErrorKind::PermissionDenied => format!("Permission denied: {}", executable),
                    _ => format!("Failed to execute command: {}", executable),
                };
                // set state
                let mut lock = lock.lock().unwrap();
                *lock = 2;
                cvar.notify_all();
                execute!(
                    io::stdout(),
                    SetForegroundColor(Color::Red),
                    Print(&*feedback),
                    ResetColor,
                    Print("\n")
                )
                .unwrap();
                continue;
            }
        };
        
        let stdin = child.stdin.take().expect("Failed to open stdin");
        child::handle_child_loop(
            client.clone(),
            child,
            stdin,
            command_lock.clone(),
            command_feedback.clone(),
            tx.clone(),
        )
        .await;

        // Wait for feedback from the child process
        if let Ok(feedback) = rx.recv() {
            println!("Feedback from child: {}", feedback);
        }
    }

    Ok(())
}
