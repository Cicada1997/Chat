pub mod error;

//              //
//  re-exports  //
//              //
pub use error::Error;

use {
    tokio::{
        io::{ AsyncWriteExt, AsyncBufReadExt, BufReader },
        net::{
            TcpListener,
            tcp::OwnedReadHalf
        },
        sync::mpsc,
    },

    futures::StreamExt,
};

#[tokio::main]
async fn main() -> Result<(), Error> {

    let mut server = Server::new("127.0.0.1:5225");

    server.listen().await;

    Ok(())
}

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
    channel: Channel<String>,
}

impl Server {
    pub fn new(addr: &str) -> Self {
        Self {
            addr:    addr.to_owned(),
            channel: Channel::new(60),
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

                    clients.push(writer);

                    tokio::spawn(handle_connection(reader, addr, self.channel.sender.clone()));
                },
                Some(msg) = self.channel.receiver.recv() => {
                    let byte_msg = msg.as_bytes();



                    clients = futures::stream::iter(clients).filter_map(|mut client| async move {
                        match client.write_all(byte_msg).await {
                            Ok(()) => Some(client),
                            Err(_) => None,
                        }
                    })
                    .collect::<Vec<_>>()
                    .await;
                }
            }
        }
    }
}

pub async fn handle_connection(reader_half: OwnedReadHalf, addr: std::net::SocketAddr, sender: mpsc::Sender<String>) {
    eprintln!("[ {addr} CONNECTED ] :)");
    let mut reader = BufReader::new(reader_half);

    let mut line = String::new();
    while let Ok(code) = reader.read_line(&mut line).await {
        if code == 0 {
            eprintln!("[ {addr} DISCONNECTED ] :(");
            break;
        }

        line = line.strip_suffix('\n').unwrap_or(&line).to_string();

        println!("[ {addr} SENT A MESSAGE ] '{line}'");

        if let Some(e) = sender.send(line.clone()).await.err() {
            eprintln!("[ UNABLE TO BROADCAST ] {e}");
            break;
        }

        line.clear();
    }
}
