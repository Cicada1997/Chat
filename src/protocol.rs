use crate::{
    Error,
    auth::UserDetails,
};

use {
    std::net::SocketAddr,
     
    tokio::{
        io::{ AsyncBufReadExt, BufReader },
        net::tcp::OwnedReadHalf,
    },

    serde::{Deserialize, Serialize}
};

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub author_id: u32,
    pub content: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ServerPacket {
    Connect {
        user: UserDetails,
        addr: SocketAddr,
    },
    Disconnect {
        reason: String,
        addr: SocketAddr,
    },
    NewMessage {
        author_id: u32,
        username: Option<String>,
        content: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientPacket {
    Disconnect,
    SendMessage {
        content: String,
        channel_id: u32,
    },
    TokenLogin { token: String },
    Login {
        username: String,
        password: String,
    },
}

pub async fn next_packet(reader: &mut BufReader<OwnedReadHalf>) -> Result<ClientPacket, Error> {
    let mut line = String::new();
    let code = reader.read_line(&mut line).await?;

    if code == 0 {
        return Ok(ClientPacket::Disconnect);
    }

    line = line.strip_suffix('\n').unwrap_or(&line).to_string();
    let Ok(client_packet) = serde_json::from_str::<ClientPacket>(&line) else {
        return Err("ClientPacketParseError".into());
    };

    Ok(client_packet)
}

#[macro_export]
macro_rules! try_next_packet {
    ($reader:expr, $addr:ident) => {
        {
            use crate::protocol::next_packet;
            let packet;

            'trying: loop {
                match next_packet($reader).await {
                    Ok(cp) => {
                        packet = cp;
                        break 'trying;
                    },
                    Err(e) => {
                        eprintln!("[ WARNING ] malformed packet sent from {}: {e}", $addr);
                        continue;
                    },
                };
            }

            packet
        }
    }
}
