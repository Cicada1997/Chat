use crate::{
    Error,
    config::Config,
    handler::Handler,
    protocol::ServerPacket,
};

use {
    std::{
        collections::HashMap,
        sync::Arc,
    },
    tokio::{
        io::AsyncWriteExt,
        net::{
            TcpListener,
            tcp::OwnedWriteHalf,
        },
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
    conf: Arc<Config>,

    broadcast: Channel<ServerPacket>,
}

static MAX_ATTEMPTS: usize = 3;

pub async fn send_data(writer: &mut OwnedWriteHalf, data: &String) -> Result<(), Error> {
    'try_loop: for attempt in 1..=MAX_ATTEMPTS {
        if let Some(err) = writer.write_all(data.as_bytes()).await.err() {
            eprintln!("[ FAILED TO SEND PACKET ] reason (attempt: {attempt}): {err}");
            if attempt == MAX_ATTEMPTS { return Err(Box::new(err)) }
        }

        let _ = writer.flush().await;
        break 'try_loop;
    }

    Ok(())
}

impl Server {
    pub fn new(conf: Arc<Config>) -> Self {
        Self {
            conf,

            broadcast: Channel::new(60),
        }
    }

    pub async fn listen(&mut self) {
        let addr = format!("{}:{}", self.conf.chat.ip, self.conf.chat.port.unwrap_or(Config::default().chat.port.unwrap()));
        let listener = TcpListener::bind(addr).await.unwrap();
        let mut clients = Vec::new();
        let mut client_cache = HashMap::new();
        let mut messages: Vec<String> = Vec::new();

        let handler = Arc::new(Handler::new(self.conf.clone(), self.broadcast.sender.clone()));

        'listning: loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => break 'listning,
                res = listener.accept() => {
                    let (socket, addr) = res.unwrap();
                    let (reader, writer) = socket.into_split();

                    clients.push((writer, addr));

                    {
                        let handler_c = handler.clone();
                        tokio::spawn( async move {
                            handler_c.handle_client(reader, addr).await;
                        });
                    }
                },

                Some(packet) = self.broadcast.receiver.recv() => {
                    match packet.clone() {
                        ServerPacket::Connect { user, addr } => {
                            client_cache.insert(user.user_id, (addr, user.clone()));

                            for (writer, addr_match) in clients.iter_mut() {
                                if *addr_match == addr {
                                    for pack in messages.split_at(std::cmp::max(messages.len() as isize - 20, 0) as usize).1.iter() {
                                        send_data(writer, pack).await.unwrap();
                                    }
                                }
                            }
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

                            messages.push(json_packet.clone());

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
