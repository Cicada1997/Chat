use std::collections::HashMap;

use tokio::net::{tcp::OwnedWriteHalf, unix::SocketAddr};

use crate::{
    auth::UserDetails, config::Config, handler, protocol::ServerPacket
};

use {
    std::sync::Arc,
    tokio::{
        net::TcpListener,
        sync::mpsc,
    },
};

struct Channel<T> {
    sender: mpsc::Sender<T>,
    receiver: mpsc::Receiver<T>,
}

impl <T>Channel<T> {
    pub fn new(buffer: usize) -> Self {
        let ch = mpsc::channel(buffer);

        Self { sender: ch.0, receiver: ch.1 }
    }
}

pub struct Server {
    // addr: String,
    conf: Arc<Config>,

    broadcast: Channel<ServerPacket>,
}

// pub struct ChachedUser {
//     writer: OwnedWriteHalf,
//     addr: SocketAddr,
//     cache: Option<UserDetails>,
// }

impl Server {
    pub fn new(conf: Arc<Config>) -> Self {
        Self {
            // addr: format!("{}:{}", conf.chat.ip,
            // conf.chat.port.unwrap_or(Config::default().chat.port)),
            conf,

            broadcast: Channel::new(60),
        }
    }

    pub async fn listen(&mut self) {
        let addr = format!("{}:{}", self.conf.chat.ip, self.conf.chat.port.unwrap_or(Config::default().chat.port.unwrap()));
        let listener = TcpListener::bind(addr).await.unwrap();
        let mut clients = Vec::new();
        let mut client_cache = HashMap::new();

        'listning: loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => break 'listning,
                res = listener.accept() => {
                    let (socket, addr) = res.unwrap();
                    let (reader, writer) = socket.into_split();

                    clients.push((writer, addr));

                    tokio::spawn(handler::handle_client(reader, addr, self.broadcast.sender.clone(), self.conf.clone()));
                },

                Some(packet) = self.broadcast.receiver.recv() => {
                    match packet.clone() {
                        ServerPacket::Connect { user, addr } => {
                            client_cache.insert(user.user_id, (addr, user));
                        }

                        ServerPacket::NewMessage { username, author_id, content } => {
                            let mut name = username;
                            if name.is_none() {
                                if let Some((_addr, user)) = client_cache.get(&author_id) {
                                    name = Some(user.username.clone());
                                }
                            }

                            let pack = ServerPacket::NewMessage { username: name, author_id, content };

                            let json_packet = match serde_json::to_string(&pack) {
                                Ok(json_packet) => json_packet + "\n",
                                Err(_) => continue,
                            };

                            for (writer, _) in clients.iter_mut() {
                                let _ = writer.try_write(json_packet.as_bytes());
                            }
                        }

                        ServerPacket::Disconnect { reason, addr } => {
                            let mut disconnect_idx = None;
                            let json_packet = match serde_json::to_string(&packet.clone()) {
                                Ok(json_packet) => json_packet + "\n",
                                Err(_) => continue,
                            };
                            for (idx, (client_writer, client_addr)) in clients.iter().enumerate() {
                                if *client_addr == addr {
                                    disconnect_idx = Some(idx);
                                    let _ = client_writer.try_write(json_packet.as_bytes());
                                    break
                                }
                            }

                            if let Some(idx) = disconnect_idx {
                                clients.remove(idx);
                                println!("[ {addr} DISCONNECTED ] Server disconnected client for reason: {reason}");
                            } else {
                                println!("[ FAILED DISCONNECT ] Tried to disconnect client for reason: '{reason}' but failed.");
                            }
                        }
                    }
        }   }   }
            for (writer, addr) in clients.iter_mut() {
                let _ = writer.try_write(
                    serde_json::to_string(
                        &ServerPacket::Disconnect {
                            reason: String::from("Server is closing."),
                            addr:   *addr
                        }
                    )
                    .unwrap()
                    .as_bytes());
        }
    }
}
