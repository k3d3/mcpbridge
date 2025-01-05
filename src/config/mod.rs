use serde::Deserialize;
use std::collections::HashMap;
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub bridge: Bridge,
    #[serde(rename = "mcpServers")]
    pub mcp_servers: HashMap<String, ServerConfig>,
}

#[derive(Debug, Deserialize)]
pub struct Bridge {
    pub listen: String,
    pub api_key: String,
    pub tls: TlsConfig,
}

impl Bridge {
    pub fn socket_addr(&self) -> Result<SocketAddr, Box<dyn std::error::Error>> {
        Ok(self.listen.parse()?)
    }
}

#[derive(Debug, Deserialize)]
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub command: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}
