use crate::{
    auth::auth_user,
    config::Config,
    protocol::{
        ClientPacket, ServerPacket
    }, try_next_packet
};

use {
    std::sync::Arc,
    tokio::{
        io::BufReader,
        net::tcp::OwnedReadHalf,
        sync::mpsc,
    },
};

pub async fn handle_client(reader_half: OwnedReadHalf, addr: std::net::SocketAddr, sender: mpsc::Sender<ServerPacket>, conf: Arc<Config>) {
    let mut reader = BufReader::new(reader_half);
    println!("[ {addr} CONNECTED ] awaiting login credentials...");
    let user = match auth_user(&mut reader, addr, &conf.auth.url).await {
        Ok(u) => u,
        Err(e) => {
            println!("[ {addr} INFO ] Failed login: {e}");
            let reason = format!("Unable to authenticate ({e})");
            let _ = sender
                .send( ServerPacket::Disconnect { reason, addr })
                .await
                .unwrap();
            return;
        }
    };

    println!("[ {addr} AUTHENTICATED ] successfully as {}", user.username);
    let _ = sender
        .send( ServerPacket::Connect { addr, user: user.clone() } )
        .await
        .unwrap();

    'connected: loop {
        let client_packet = try_next_packet!(&mut reader, addr);

        match client_packet {
            ClientPacket::Disconnect => {
                eprintln!("[ user {} ({}) DISCONNECTED ] Client sent disconnect signal", user.user_id, user.username);
                break 'connected;
            }

            ClientPacket::SendMessage { content, .. } => {
                if content.len() == 0 { return; }

                println!("[ {addr} SENT A MESSAGE ] '{content}'");
                let msg_packet = ServerPacket::NewMessage { username: Some(user.username.clone()), author_id: user.user_id, content: content };

                if let Some(e) = sender.send(msg_packet).await.err() {
                    eprintln!("[ UNABLE TO BROADCAST ] {e}");
                    break;
                }
            }
            _ => {}
        }
    }
}
