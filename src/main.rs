mod config;
mod server;
mod websocket;
mod tls;

use std::env;
use std::fs;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <config_file>", args[0]);
        std::process::exit(1);
    }

    let config_path = &args[1];
    let config_content = fs::read_to_string(config_path)?;
    let config: config::Config = serde_json::from_str(&config_content)?;
    
    // Load TLS configuration
    let tls_config = tls::load_tls_config(
        &config.bridge.tls.cert_path,
        &config.bridge.tls.key_path,
    )?;
    let acceptor = TlsAcceptor::from(Arc::new(tls_config));
    
    let addr = config.bridge.socket_addr()?;
    let listener = TcpListener::bind(addr).await?;
    println!("Secure WebSocket server listening on: {}", config.bridge.listen);

    let config = Arc::new(config);

    while let Ok((stream, peer_addr)) = listener.accept().await {
        println!("Incoming connection from: {}", peer_addr);
        let acceptor = acceptor.clone();
        let config = Arc::clone(&config);
        
        tokio::spawn(async move {
            match acceptor.accept(stream).await {
                Ok(tls_stream) => {
                    websocket::handle_connection(tls_stream, config).await;
                }
                Err(e) => {
                    eprintln!("TLS error from {}: {}", peer_addr, e);
                }
            }
        });
    }

    Ok(())
}
