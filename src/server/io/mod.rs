mod handlers;
mod message_loop;

use futures_util::stream::{SplitSink, SplitStream};
use tokio::net::TcpStream;
use tokio::process::Child;
use tokio::sync::mpsc;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;

use handlers::{spawn_stdout_handler, spawn_stderr_handler};
use message_loop::run_message_loop;

pub async fn handle_process_io(
    write: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
    read: &mut SplitStream<WebSocketStream<TcpStream>>,
    child: &mut Child,
) -> Result<(), Box<dyn std::error::Error>> {
    let stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let (tx, rx) = mpsc::channel::<String>(32);
    let tx_clone = tx.clone();

    let stdout_task = spawn_stdout_handler(stdout, tx);
    let stderr_task = spawn_stderr_handler(stderr, tx_clone);

    run_message_loop(write, read, rx, stdin).await?;

    let _ = child.kill().await;
    let _ = stdout_task.abort();
    let _ = stderr_task.abort();

    Ok(())
}
