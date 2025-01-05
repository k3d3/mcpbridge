use crate::config::Config;
use crate::websocket::messages::ConnectionMessage;
use crate::websocket::connection::get_first_message;
use crate::websocket::error::send_error;
use crate::websocket::list::handle_list_request;
use crate::websocket::connect::handle_connect_request;
use futures_util::StreamExt;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::accept_async;

pub async fn handle_connection<S>(stream: S, config: Arc<Config>) 
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut ws_stream = match accept_async(stream).await {
        Ok(ws_stream) => ws_stream,
        Err(e) => {
            eprintln!("Error during WebSocket handshake: {}", e);
            return;
        }
    };

    let first_message = get_first_message(&mut ws_stream).await;
    dbg!(&first_message);
    let (mut write, mut read) = ws_stream.split();

    match first_message {
        Ok(ConnectionMessage::List { list: true, key }) => {
            handle_list_request(&mut write, &config, &key).await;
        }
        Ok(ConnectionMessage::Connect { connect, key }) => {
            handle_connect_request(&mut write, &mut read, &config, &connect, &key).await;
        }
        _ => {
            send_error(&mut write, "Invalid request").await;
        }
    }
}
