use crate::websocket::messages::ConnectionMessage;
use futures_util::StreamExt;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::WebSocketStream;

pub async fn get_first_message<S>(
    stream: &mut WebSocketStream<S>
) -> Result<ConnectionMessage, &'static str>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    if let Some(Ok(msg)) = stream.next().await {
        if let Ok(text) = msg.into_text() {
            match serde_json::from_str::<ConnectionMessage>(&text) {
                Ok(msg) => Ok(msg),
                Err(_) => Err("Invalid message format")
            }
        } else {
            Err("First message must be text")
        }
    } else {
        Err("No message received")
    }
}
