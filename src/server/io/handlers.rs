use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

pub fn spawn_stdout_handler(
    stdout: impl tokio::io::AsyncRead + Unpin + Send + 'static,
    tx: Sender<String>,
) -> JoinHandle<()> {
    let stdout_reader = BufReader::new(stdout);
    let mut stdout_lines = stdout_reader.lines();
    tokio::spawn(async move {
        while let Ok(Some(line)) = stdout_lines.next_line().await {
            if tx.send(line).await.is_err() {
                break;
            }
        }
    })
}

pub fn spawn_stderr_handler(
    stderr: impl tokio::io::AsyncRead + Unpin + Send + 'static,
    tx: Sender<String>,
) -> JoinHandle<()> {
    let stderr_reader = BufReader::new(stderr);
    let mut stderr_lines = stderr_reader.lines();
    tokio::spawn(async move {
        while let Ok(Some(line)) = stderr_lines.next_line().await {
            if tx.send(line).await.is_err() {
                break;
            }
        }
    })
}
