
use serde_json::json;

use crate::{
    protocol::ClientPacket,
    try_next_packet,
    Error,
};

use {
    tokio::{
        io::BufReader,
        net::tcp::OwnedReadHalf,
    },

    serde::{ Deserialize, Serialize },
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserDetails {
    pub user_id:    u32,
    pub username:   String,
    pub admin:      bool,
}

pub async fn auth_by_token(token: String, auth_url: &str) -> Result<UserDetails, Error> {
    let res = reqwest::Client::new()
        .post(format!("https://{auth_url}/token-login"))
        .json(&format!("{token}"))
        .send()
        .await?;

    if res.status().is_success() {
        return Ok(res.json::<UserDetails>().await?);
    }

    return Err(format!("Unable to authenticate: {}", res.status()).into())
}

pub async fn auth_by_username(username: String, password: String, auth_url: &str) -> Result<UserDetails, Error> {
    let res = reqwest::Client::new()
        .post(format!("https://{auth_url}/login"))
        .json(&json!({
            "username": username,
            "hashword": password,
        }))
    .send()
        .await?;

    if res.status().is_success() {
        let token = res.json::<String>().await?;
        return auth_by_token(token, auth_url).await;
    }

    return Err(format!("Unable to authenticate: {}", res.status()).into())
}

pub async fn auth_user(reader: &mut BufReader<OwnedReadHalf>, addr: std::net::SocketAddr, auth_url: &str) -> Result<UserDetails, Error> {
    loop {
        let packet = try_next_packet!(reader, addr);
        match packet {
            ClientPacket::TokenLogin { token } => {
                return auth_by_token(token, auth_url).await;
            }

            ClientPacket::Login { username, password } => {
                return auth_by_username(username, password, auth_url).await;
            }

            ClientPacket::Disconnect => {
                return Err("User disconnected".into());
            }

            _ => {
                eprintln!("[ WARNING ] {addr} tried to send authorized packet without being authenticated {packet:?}")
            }
        }
    }
}
