use crate::toolkit::portscan::sweeper::PortScanner;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::time::sleep;

pub async fn terminal_thread(
    sessnum: i32,
    ip: String,
    data: PortScanner,
    previous_output: Arc<Mutex<String>>,
) -> Result<(), i32> {
    println!("[TTS:{}] Starting terminal thread", sessnum);
    println!("[TTS:{}] IP: {}", sessnum, ip);
    println!("[TTS:{}] Data: {:?}", sessnum, data);

    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    let (tx, mut rx): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel(1);

    let mut child = match Command::new("sh")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            println!("[!!!] Failed to start subprocess: {}", e);
            return Err(1);
        }
    };

    let mut child_stdin = child.stdin.take().expect("Failed to open stdin");
    let child_stdout = child.stdout.take().expect("Failed to open stdout");
    let child_stderr = child.stderr.take().expect("Failed to open stderr");

    // Task to handle reading from stdout
    let previous_output_clone = Arc::clone(&previous_output);
    let output_task = tokio::spawn(async move {
        let mut reader = BufReader::new(child_stdout);
        let mut line = String::new();

        while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
            // println!("[...] {}", line);
            let mut prev_output = previous_output_clone.lock().await;
            prev_output.push_str(&line);
            line.clear();
        }
    });

    // Task to handle reading from stderr
    let previous_output_clone = Arc::clone(&previous_output);
    let error_task = tokio::spawn(async move {
        let mut reader = BufReader::new(child_stderr);
        let mut line = String::new();

        while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
            // eprintln!("[!!!] {}", line);
            let mut prev_output = previous_output_clone.lock().await;
            prev_output.push_str(&line);
            line.clear();
        }
    });

    // Task to handle sending input to the child process
    let input_task = tokio::spawn(async move {
        while let Some(command) = rx.recv().await {
            if let Err(e) = child_stdin.write_all(command.as_bytes()).await {
                eprintln!("[!!!] Failed to write to subprocess stdin: {}", e);
                break;
            }
            if let Err(e) = child_stdin.flush().await {
                eprintln!("[!!!] Failed to flush subprocess stdin: {}", e);
                break;
            }
        }
    });

    let mut command_typed = false;
    let mut command_wait = false;
    let command_wait_keyword: [&str; 10] = [
        "update",
        "upgrade",
        "install",
        "remove",
        "uninstall",
        "purge",
        "autoremove",
        "dist-upgrade",
        "full-upgrade",
        "clean",
    ];
    loop {
        // Check if there's output to display
        let mut opt_before = String::new();
        if command_wait {
            println!("Waiting for progressed command to finish, can take few minutes...");
        }
        loop {
            if !command_typed {
                break;
            }
            if command_wait {
                sleep(Duration::from_secs(10)).await;
            } else {
                sleep(Duration::from_millis(200)).await;
            }
            let mut prev_output = previous_output.lock().await;
            opt_before.push_str(&*prev_output);
            
            if !prev_output.is_empty() {
                prev_output.clear();
            } else {
                command_wait = false;
                command_typed = false;
            }
        }

        // Allow user to input commands without being interrupted
        println!("result: \n{} \n$:", opt_before);
        line.clear();

        let read_result = reader.read_line(&mut line).await;
        match read_result {
            Ok(0) => break, // End of input (Ctrl+D or similar)
            Ok(_) => {
                let command_input = line.trim();
                if command_input.eq_ignore_ascii_case("end") {
                    println!("Exit command received. Exiting terminal thread.");
                    break;
                }

                if !command_input.is_empty() {
                    command_typed = true;

                    // Check if the command requires waiting for a keyword
                    for keyword in command_wait_keyword.iter() {
                        if command_input.contains(keyword) {
                            command_wait = true;
                            break;
                        }
                    }

                    if let Err(e) = tx.send(format!("{}\n", command_input)).await {
                        eprintln!("[!!!] Failed to send command to subprocess: {}", e);
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("[!!!] Failed to read user input: {}", e);
                break;
            }
        }
    }

    drop(tx);

    if let Err(e) = child.wait().await {
        eprintln!("[TTS:{}] Failed to wait on child process: {}", sessnum, e);
    }

    if let Err(e) = output_task.await {
        eprintln!("[!!!] {:?}", e);
    }
    if let Err(e) = error_task.await {
        eprintln!("[!!!] {:?}", e);
    }
    if let Err(e) = input_task.await {
        eprintln!("[!!!] {:?}", e);
    }

    println!("[TTS:{}] Command session finished.", sessnum);

    Ok(())
}
