use crate::config::ServerConfig;
use crate::websocket::messages::StatusMessage;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::SinkExt;
use std::process::Stdio;
use tokio::net::TcpStream;
use tokio::process::Command;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;

use super::io::handle_process_io;

pub async fn spawn_server(
    write: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
    read: &mut SplitStream<WebSocketStream<TcpStream>>,
    server_config: &ServerConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut child = Command::new(&server_config.command)
        .args(&server_config.args)
        .envs(&server_config.env)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let status = StatusMessage {
        status: "connected".to_string(),
    };
    write.send(Message::Text(serde_json::to_string(&status)?)).await?;

    handle_process_io(write, read, &mut child).await?;

    Ok(())
}
