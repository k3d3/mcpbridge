use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ConnectionMessage {
    List {
        list: bool,
        key: String,
    },
    Connect {
        connect: String,
        key: String,
    },
}

#[derive(Debug, Serialize)]
pub struct ErrorMessage {
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct StatusMessage {
    pub status: String,
}
