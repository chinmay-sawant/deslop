use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Credentials {
    pub api_key: String,
    pub port: u16,
}

pub struct Order {
    pub price: f64,
    pub total: i64,
}

pub struct ServerConfig {
    pub ssl_enabled: bool,
    pub cert: Option<String>,
}