use crate::{
    handler,
    protocol::ServerPacket,
};

use {
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
    addr: String,
    broadcast: Channel<ServerPacket>,
}

impl Server {
    pub fn new(addr: &str) -> Self {
        Self {
            addr: addr.to_owned(),
            broadcast: Channel::new(60),
        }
    }

    pub async fn listen(&mut self) {
        let listener = TcpListener::bind(&self.addr).await.unwrap();

        let mut clients = Vec::new();

        'listning: loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => break 'listning,
                res = listener.accept() => {
                    let (socket, addr) = res.unwrap();
                    let (reader, writer) = socket.into_split();

                    clients.push((writer, addr));

                    tokio::spawn(handler::handle_client(reader, addr, self.broadcast.sender.clone()));
                },

                Some(packet) = self.broadcast.receiver.recv() => {
                    let json_packet = match serde_json::to_string(&packet) {
                        Ok(json_packet) => json_packet + "\n",
                        Err(_) => continue,
                    };

                    match packet {
                        ServerPacket::NewMessage(..) => {

                            for (writer, _) in clients.iter_mut() {
                                let _ = writer.try_write(json_packet.as_bytes());
                            }
                        }

                        ServerPacket::Disconnect { reason, addr } => {
                            let mut disconnect_idx = None;
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
    }   }   }   }   }
}
