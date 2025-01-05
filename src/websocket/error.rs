use crate::websocket::messages::ErrorMessage;
use futures_util::SinkExt;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;

pub async fn send_error<S>(
    write: &mut futures_util::stream::SplitSink<WebSocketStream<S>, Message>,
    error: &str,
) where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let error = ErrorMessage {
        error: error.to_string()
    };
    let _ = write
        .send(Message::Text(serde_json::to_string(&error).unwrap()))
        .await;
}
