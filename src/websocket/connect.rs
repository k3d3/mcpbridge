use crate::config::Config;
use crate::websocket::error::send_error;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;

pub async fn handle_connect_request<S>(
    write: &mut SplitSink<WebSocketStream<S>, Message>,
    read: &mut SplitStream<WebSocketStream<S>>,
    config: &Config,
    connect: &str,
    key: &str,
) where
    S: AsyncRead + AsyncWrite + Unpin,
{
    if key != config.bridge.api_key {
        send_error(write, "Invalid API key").await;
        return;
    }

    if let Some(server_config) = config.mcp_servers.get(connect) {
        // Start the server process
        let mut child = match tokio::process::Command::new(&server_config.command)
            .args(&server_config.args)
            .envs(&server_config.env)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn() {
                Ok(child) => child,
                Err(e) => {
                    send_error(write, &format!("Failed to start server: {}", e)).await;
                    return;
                }
            };

        // Get process I/O handles
        let mut stdin = child.stdin.take().expect("Failed to get stdin");
        let mut stdout = child.stdout.take().expect("Failed to get stdout");
        let mut stderr = child.stderr.take().expect("Failed to get stderr");

        // Set up communication channel for stdout/stderr
        let (tx, mut rx) = mpsc::channel::<String>(32);
        let tx_clone = tx.clone();

        // Spawn stdout handler
        let stdout_task = tokio::spawn(async move {
            let mut buffer = [0u8; 1024];
            let mut current_line = Vec::new();
            
            loop {
                match stdout.read(&mut buffer).await {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        for &byte in &buffer[..n] {
                            if byte == b'\n' {
                                if let Ok(line) = String::from_utf8(current_line.clone()) {
                                    if let Err(_) = tx.send(line).await {
                                        return;
                                    }
                                }
                                current_line.clear();
                            } else {
                                current_line.push(byte);
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        // Spawn stderr handler
        let stderr_task = tokio::spawn(async move {
            let mut buffer = [0u8; 1024];
            let mut current_line = Vec::new();
            
            loop {
                match stderr.read(&mut buffer).await {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        for &byte in &buffer[..n] {
                            if byte == b'\n' {
                                if let Ok(line) = String::from_utf8(current_line.clone()) {
                                    if let Err(_) = tx_clone.send(line).await {
                                        return;
                                    }
                                }
                                current_line.clear();
                            } else {
                                current_line.push(byte);
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        // Send connected status
        let status = crate::websocket::messages::StatusMessage {
            status: "connected".to_string(),
        };
        if let Err(e) = write.send(Message::Text(serde_json::to_string(&status).unwrap())).await {
            eprintln!("Error sending status: {}", e);
            return;
        }

        // Main message loop
        loop {
            tokio::select! {
                Some(line) = rx.recv() => {
                    println!("Got message on rx: {line}");
                    if let Err(e) = write.send(Message::Text(line)).await {
                        eprintln!("Error sending output: {}", e);
                        break;
                    }
                }
                Some(result) = read.next() => {
                    println!("Got message on read");
                    match result {
                        Ok(msg) => {
                            if let Ok(text) = msg.into_text() {
                                println!("Got message: {text}");
                                if let Err(_) = stdin.write_all(text.as_bytes()).await {
                                    break;
                                }
                                if let Err(_) = stdin.write_all(b"\n").await {
                                    break;
                                }
                            }
                        }
                        Err(_) => break,
                    }
                }
                else => break,
            }
        }

        // Clean up
        let _ = child.kill().await;
        let _ = stdout_task.abort();
        let _ = stderr_task.abort();
    } else {
        send_error(write, "Unknown server type").await;
    }
}