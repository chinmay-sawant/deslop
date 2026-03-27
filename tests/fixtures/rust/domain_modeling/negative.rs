use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub display_name: String,
}

pub struct OrderAmount {
    pub cents: i64,
}

pub enum SslConfig {
    Disabled,
    EnabledWithCert(String),
}