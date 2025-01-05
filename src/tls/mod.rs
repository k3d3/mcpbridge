use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};

pub fn load_tls_config(cert_path: &str, key_path: &str) -> Result<ServerConfig, Box<dyn std::error::Error>> {
    // Load certificate chain
    let cert_file = File::open(cert_path)?;
    let mut reader = BufReader::new(cert_file);
    let certs = certs(&mut reader)?
        .into_iter()
        .map(Certificate)
        .collect();

    // Load private key
    let key_file = File::open(key_path)?;
    let mut reader = BufReader::new(key_file);
    let keys = pkcs8_private_keys(&mut reader)?;
    if keys.is_empty() {
        return Err("No private keys found".into());
    }
    let key = PrivateKey(keys[0].clone());

    // Create TLS config
    let mut config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;

    // Enable key logging for wireshark analysis
    config.key_log = Arc::new(tokio_rustls::rustls::KeyLogFile::new());

    Ok(config)
}