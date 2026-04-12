use crate::{
    try_next_packet,

    protocol::{
        ServerPacket,
        ClientPacket,
        Message,
    },
    auth::auth_user,
};

use {
    tokio::{
        io::BufReader,
        net::tcp::OwnedReadHalf,
        sync::mpsc,
    },
};

pub async fn handle_client(reader_half: OwnedReadHalf, addr: std::net::SocketAddr, sender: mpsc::Sender<ServerPacket>) {
    let mut reader = BufReader::new(reader_half);
    println!("[ {addr} CONNECTED ] awaiting login credentials...");

    let user = match auth_user(&mut reader, addr).await {
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

    'connected: loop {
        let client_packet = try_next_packet!(&mut reader, addr);

        match client_packet {
            ClientPacket::Disconnect => {
                eprintln!("[ user {} ({}) DISCONNECTED ] Client sent disconnect signal", user.user_id, user.username);
                break 'connected;
            }

            ClientPacket::SendMessage { content, .. } => {
                println!("[ {addr} SENT A MESSAGE ] '{content}'");
                let msg_packet = ServerPacket::NewMessage( Message { author_id: user.user_id, content: content });

                if let Some(e) = sender.send(msg_packet).await.err() {
                    eprintln!("[ UNABLE TO BROADCAST ] {e}");
                    break;
                }
            }
            _ => {}
        }
    }
}
