use std::io::{self, BufRead, BufReader, ErrorKind, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use crossterm::execute;
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};

use crate::http::openai::ClientAi;
use crate::log::printlg;

pub async fn handle_child_loop(
    client: Arc<Mutex<ClientAi>>,
    mut child: Child,
    mut stdin: ChildStdin,
    command_lock: Arc<(Mutex<i32>, Condvar)>,
    command_feedback: Arc<Mutex<String>>,
    tx: Sender<String>,
) {
    let mut commander = String::new();

    let stdout = child.stdout.take().expect("Failed to open stdout");
    let stderr = child.stderr.take().expect("Failed to open stderr");

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    let command_lock_clone_stdout = Arc::clone(&command_lock);
    let command_lock_clone_stderr = Arc::clone(&command_lock);
    let command_feedback_clone_stdout = Arc::clone(&command_feedback);
    let command_feedback_clone_stderr = Arc::clone(&command_feedback);
    let tx_clone_stdout = tx.clone();
    let tx_clone_stderr = tx.clone();

    // List of keywords indicating completion of installation
    let completion_keywords = ["installation complete", "successfully installed", "done"];

    let stdout_thread = thread::spawn(move || {
        let mut collected_output = Vec::new();
        for line in stdout_reader.lines() {
            let line = line.expect("Failed to read line");
            collected_output.push(line.clone());
            println!("stderr: {}", line); // Debug print
            execute!(
                io::stdout(),
                SetForegroundColor(Color::Red),
                Print(&line),
                ResetColor,
                Print("\n")
            )
            .unwrap();

            // Check for completion keywords
            if completion_keywords
                .iter()
                .any(|&keyword| line.contains(keyword))
            {
                println!("Installation process completed: {}", line);
                break;
            }
        }

        // Join all collected lines into a single string
        let feedback = collected_output.join("\n");
        {
            let mut feedback_lock = command_feedback_clone_stdout.lock().unwrap();
            *feedback_lock = feedback.clone();
            println!("Feedback set to: {}", *feedback_lock); // Debug print
        }

        let (lock, cvar) = &*command_lock_clone_stdout;
        let mut lock = lock.lock().unwrap();

        // Check if the command is an installation process
        let install_keywords = ["install", "apt-get", "yum", "brew", "pip", "npm"];
        let is_installation = install_keywords
            .iter()
            .any(|&keyword| feedback.contains(keyword));

        if is_installation {
            println!("Installation process detected, setting state to 99");
            *lock = 99;
            cvar.notify_all();
        } else {
            let (lock, cvar) = &*command_lock_clone_stdout;
            let mut lock = lock.lock().unwrap();
            *lock = 12; // No error, set to next command
            cvar.notify_all();
        }
        tx_clone_stdout.send(feedback).unwrap();
    });

    let stderr_thread = thread::spawn(move || {
        let mut collected_output = Vec::new();
        for line in stderr_reader.lines() {
            let line = line.expect("Failed to read line");
            collected_output.push(line.clone());
            println!("stderr: {}", line); // Debug print
            execute!(
                io::stdout(),
                SetForegroundColor(Color::Red),
                Print(&line),
                ResetColor,
                Print("\n")
            )
            .unwrap();

            // Check for completion keywords
            if completion_keywords
                .iter()
                .any(|&keyword| line.contains(keyword))
            {
                println!("Installation process completed: {}", line);
                break;
            }
        }

        // Join all collected lines into a single string
        let feedback = collected_output.join("\n");
        {
            let mut feedback_lock = command_feedback_clone_stderr.lock().unwrap();
            *feedback_lock = feedback.clone();
            println!("Feedback set to: {}", *feedback_lock); // Debug print
        }

        let (lock, cvar) = &*command_lock_clone_stderr;
        let mut lock = lock.lock().unwrap();
        *lock = 12; // No error, set to next command
        cvar.notify_all();
        tx_clone_stderr.send(feedback).unwrap();
    });

    let mut looplock = true;

    loop {
        // Handle command feedback and state changes
        let (lock, cvar) = &*command_lock;
        let mut state = lock.lock().unwrap();
        match *state {
            12 => {
                looplock = false;
                let msg = match command_feedback.lock() {
                    Ok(msg) => match msg.clone().len() {
                        0 => "empty".to_string(),
                        _ => msg.clone(),
                    },
                    Err(_) => "hm?".to_string(),
                };
                println!("Sub State 2: Command received, feedback {}", msg);
                // Handle the feedback and continue processing
                if msg == "empty" {
                    // Handle empty feedback case
                    println!("Feedback is empty, continuing...");
                    // Update the state to continue processing
                    *state = 13;
                    cvar.notify_all();
                    continue;
                } else {
                    *state = 13;
                    cvar.notify_all();
                    continue;
                }
            }
            13 => {
                // exit
                commander = "exit".to_string();
                looplock = false;
                *state = 3;
            }
            2 => {
                // initial
                commander = "get ok".to_string();
                looplock = false;
                *state = 17;
            }
            _ => {
                continue;
            }
        }

        if looplock {
            continue;
        }
        println!("\n\nCurrent sub state: {}", *state);

        println!("Commander: {}", commander);

        let user_command = commander.trim();

        match user_command.to_lowercase().as_str() {
            "exit" => {
                child.kill().expect("Failed to kill the child process");
                break;
            }
            "change" => {
                child.kill().expect("Failed to kill the child process");
                break;
            }
            _ => {}
        }

        writeln!(stdin, "{}", user_command).expect("Failed to write to stdin");
    }

    stdout_thread.join().expect("Failed to join stdout thread");
    stderr_thread.join().expect("Failed to join stderr thread");
}
