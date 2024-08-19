use crate::http::openai::ClientAi;
use crate::toolkit::portscan::sweeper::PortScanner;
use crossterm::style::Color;
use std::io::Write;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::{mpsc, Mutex, Notify};
use tokio::time::{sleep, timeout, Instant};

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

    let notify = Arc::new(Notify::new());
    let stdout_notify = Arc::clone(&notify);
    let stderr_notify = Arc::clone(&notify);

    let previous_output_clone = Arc::clone(&previous_output);

    let output_task = tokio::spawn(async move {
        let mut reader = BufReader::new(child_stdout);
        let mut line = String::new();

        while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
            {
                let mut prev_output = previous_output_clone.lock().await;
                prev_output.push_str(&line);
            }
            eprintln!("[DEBUG] Captured stdout: {}", line); // Debugging line to verify output capture
            line.clear();
        }
        stdout_notify.notify_one(); // Notify when stdout reading is done
        eprintln!("[DEBUG] STDOUT task finished.");
    });

    let previous_output_clone = Arc::clone(&previous_output);

    let error_task = tokio::spawn(async move {
        let mut reader = BufReader::new(child_stderr);
        let mut line = String::new();

        while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
            {
                let mut prev_output = previous_output_clone.lock().await;
                prev_output.push_str(&line);
            }
            // eprintln!("[DEBUG] Captured stderr: {}", line); // Debugging line to verify output capture
            line.clear();
        }
        stderr_notify.notify_one(); // Notify when stderr reading is done
        eprintln!("[DEBUG] STDERR task finished.");
    });

    let input_task = tokio::spawn(async move {
        while let Some(command) = rx.recv().await {
            if let Err(e) = child_stdin.write_all(command.as_bytes()).await {
                println!("[!!!] Failed to write to subprocess stdin: {}", e);
                break;
            }
            if let Err(e) = child_stdin.flush().await {
                println!("[!!!] Failed to flush subprocess stdin: {}", e);
                break;
            }
        }
    });

    let mut client_ai = ClientAi::new();
    let invoke_command = client_ai
        .invoke(data.port as i32, data.desc.unwrap(), data.head.unwrap())
        .await;
    let mut invoke_command = match invoke_command {
        Ok(invoke_command) => invoke_command,
        Err(e) => {
            println!("Failed to invoke command: {:?}", e);
            return Err(1);
        }
    };
    invoke_command = invoke_command.replace("<ADDR>", &ip);
    let mut command_initial = true;
    let command_wait_keywords: [&str; 12] = [
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
        "nmap",
        "hydra",
    ];

    let mut opt_before = String::new();
    let mut commands = String::new();

    loop {
        if commands == "end" {
            println!("[TTS:{}] Exit command received. Exiting terminal thread.", sessnum);
            break;
        }

        if command_initial {
            command_initial = false;
            commands = invoke_command.clone();
        } else {
            let mut command_gen = client_ai.intruder(opt_before.clone()).await;
            command_gen = command_gen.replace("<ADDR>", &ip);
            commands = command_gen;
        }
        if commands.is_empty() {
            println!("[TTS:{}] No command generated, exiting.", sessnum);
            break;
        }

        println!("[TTS:{}] Command: {}", sessnum, commands);

        let command_input = commands.trim();
        if command_input.eq_ignore_ascii_case("end") {
            eprintln!("Exit command received. Exiting terminal thread.");
            break;
        }

        if command_input.is_empty() {
            println!(
                "[TTS:{}] Empty command, continuing to next iteration.",
                sessnum
            );
            continue;
        }

        let mut command_executed = false;

        for keyword in command_wait_keywords.iter() {
            if command_input.contains(keyword) {
                println!("Executing long-running command...");
                let result =
                    process_command(command_input, &tx, &notify, Arc::clone(&previous_output))
                        .await;
                match result {
                    Ok(output) => {
                        eprintln!("[TTS:{}] Command output: {}", sessnum, output);
                        opt_before = output; // Use this output for the next command generation
                    }
                    Err(e) => println!("Error during command execution: {:?}", e),
                }
                command_executed = true;
                break;
            }
        }

        if !command_executed {
            // If the command didn't match any long-running keywords, still process it
            println!("Executing command...");
            let result =
                process_command(command_input, &tx, &notify, Arc::clone(&previous_output)).await;
            match result {
                Ok(output) => {
                    eprintln!("[TTS:{}] Command output: {}", sessnum, output);
                    opt_before = output; // Use this output for the next command generation
                }
                Err(e) => println!("Error during command execution: {:?}", e),
            }
        }

        // Ensure the loop waits for the current command to fully process before continuing
        println!(
            "[TTS:{}] Finish executing either with error or success.",
            sessnum
        );
        if (timeout(Duration::from_secs(5), notify.notified()).await).is_err() {
            println!("[TTS:{}] No notification received. Timeout.", sessnum);
        }
        println!("[TTS:{}] Moving to the next command.", sessnum);
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

// New command processing logic
async fn process_command(
    command: &str,
    tx: &mpsc::Sender<String>,
    notify: &Arc<Notify>,
    previous_output: Arc<Mutex<String>>,
) -> Result<String, Box<dyn std::error::Error>> {
    tx.send(format!("{}\n", command)).await?;

    let mut accumulated_output = String::new();
    let mut remaining_time = Duration::from_secs(30); // Initial 30 seconds timeout
    let output_timeout = Duration::from_secs(5); // Timeout for each check

    loop {
        sleep(Duration::from_millis(100)).await;

        let mut got_output = false;

        {
            let mut prev_output = previous_output.lock().await;
            if !prev_output.is_empty() {
                accumulated_output.push_str(&prev_output);
                prev_output.clear();
                got_output = true;
            }
        }

        if got_output && remaining_time < Duration::from_secs(10) {
            remaining_time += Duration::from_secs(1); // Extend by 1 second if less than 10 seconds left
            println!("Extending timeout by 1 second. New timeout: {:?}", remaining_time);
        }

        let notify_fut = notify.notified();
        if timeout(output_timeout, notify_fut).await.is_ok() {
            break;
        }

        remaining_time = remaining_time.saturating_sub(output_timeout);

        if remaining_time.is_zero() {
            println!("Command execution timeout. Assuming completion.");
            break;
        }
    }

    // Add a small delay to ensure all output has been processed
    sleep(Duration::from_millis(500)).await;

    // Final check to capture any remaining output
    {
        let mut prev_output = previous_output.lock().await;
        if !prev_output.is_empty() {
            accumulated_output.push_str(&prev_output);
            prev_output.clear();
        }
    }

    Ok(accumulated_output)
}
