use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc::Receiver;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;

pub async fn run_message_loop(
    write: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
    read: &mut SplitStream<WebSocketStream<TcpStream>>,
    mut rx: Receiver<String>,
    mut stdin: impl AsyncWriteExt + Unpin,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        tokio::select! {
            Some(line) = rx.recv() => {
                if let Err(e) = write.send(Message::Text(line)).await {
                    eprintln!("Error sending output: {}", e);
                    break;
                }
            }
            Some(result) = read.next() => {
                match result {
                    Ok(msg) => {
                        if let Ok(text) = msg.into_text() {
                            stdin.write_all(text.as_bytes()).await?;
                            stdin.write_all(b"\n").await?;
                        }
                    }
                    Err(e) => {
                        eprintln!("WebSocket error: {}", e);
                        break;
                    }
                }
            }
            else => break,
        }
    }
    Ok(())
}
