use serde_json::json;

use crate::{
    Error,
    try_next_packet,

    protocol::ClientPacket
};

use {
    tokio::{
        io::BufReader,
        net::tcp::OwnedReadHalf,
    },

    serde::{Deserialize, Serialize}
};

pub static AUTH_URL: &'static str = "https://auth.kattmys.se";

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDetails {
    pub user_id:    u32,
    pub username:   String,
    pub admin:      bool,
}

pub async fn auth_by_token(token: String) -> Result<UserDetails, Error> {
    let res = reqwest::Client::new()
        .post(format!("{AUTH_URL}/token-login"))
        .json(&format!("{token}"))
        .send()
        .await?;

    if res.status().is_success() {
        return Ok(res.json::<UserDetails>().await?);
    }

    return Err(format!("Unable to authenticate: {}", res.status()).into())
}

pub async fn auth_by_username(username: String, password: String) -> Result<UserDetails, Error> {
    let res = reqwest::Client::new()
        .post(format!("{AUTH_URL}/login"))
        .json(&json!({
            "username": username,
            "hashword": password,
        }))
    .send()
        .await?;

    if res.status().is_success() {
        let token = res.json::<String>().await?;
        return auth_by_token(token).await;
    }

    return Err(format!("Unable to authenticate: {}", res.status()).into())
}

pub async fn auth_user(reader: &mut BufReader<OwnedReadHalf>, addr: std::net::SocketAddr) -> Result<UserDetails, Error> {
    loop {
        let packet = try_next_packet!(reader, addr);
        match packet {
            ClientPacket::TokenLogin { token } => {
                return auth_by_token(token).await;
            }

            ClientPacket::Login { username, password } => {
                return auth_by_username(username, password).await;
            }

            _ => {
                eprintln!("[ WARNING ] {addr} tried to send authorized packet without being authenticated {packet:?}")
            }
        }
    }
}
