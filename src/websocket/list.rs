use crate::config::Config;
use crate::websocket::error::send_error;
use futures_util::SinkExt;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;

pub async fn handle_list_request<S>(
    write: &mut futures_util::stream::SplitSink<WebSocketStream<S>, Message>,
    config: &Config,
    key: &str,
) where
    S: AsyncRead + AsyncWrite + Unpin,
{
    if key != config.bridge.api_key {
        send_error(write, "Invalid API key").await;
        return;
    }

    let server_list: Vec<String> = config.mcp_servers.keys().cloned().collect();
    let _ = write
        .send(Message::Text(serde_json::to_string(&server_list).unwrap()))
        .await;
}
