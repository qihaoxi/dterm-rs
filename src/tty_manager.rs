use crate::connections::Connection;
use crate::packet;
use log::info;
use log4rs;
use std::future::Future;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

struct Tty {
    sid: String,
    active: tokio::time::Duration,
    wait_ack: u32,
    recv_chan: tokio::sync::mpsc::Receiver<()>,
    lock: tokio::sync::Mutex<()>,
}

enum TtyStatus {
    Disconnected,
    Connected,
}

pub(crate) struct TtyManager {
    pub server_addr: String,
    tty_count: i32,
    tty_map: std::collections::HashMap<String, Tty>,
    lock: tokio::sync::Mutex<()>,
    status: TtyStatus,
    // sock:  tokio::net::TcpSocket,
}

impl TtyManager {
    pub(crate) fn new(addr: String) -> Self {
        Self {
            tty_count: 0,
            tty_map: std::collections::HashMap::new(),
            lock: tokio::sync::Mutex::new(()),
            status: TtyStatus::Disconnected,
            server_addr: addr,
        }
    }

    async fn connect(addr: String) -> Result<tokio::net::TcpStream, Box<dyn std::error::Error>> {
        let mut stream = TcpStream::connect(addr).await?;
        Ok(stream)
    }

    async fn destroy(&mut self, stream: &mut tokio::net::TcpStream) {
        match stream.shutdown().await {
            Ok(_) => {
                info!("shutdown success");
            }
            Err(e) => {
                info!("shutdown failed, {:?}", e);
            }
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("start connect: {}", self.server_addr);

        let mut stream = match Self::connect(self.server_addr.clone()).await {
            Ok(stream) => {
                info!("connect success");
                stream
            }
            Err(e) => {
                info!("connect failed, {:?}", e);
                return Err(e);
            }
        };

        // write register packet
        stream.write_u8(0).await?;
        stream.write_u16(34).await?;

        let mut connection = Connection::new(stream);
        loop {
            match connection.read_packet().await {
                Ok(Some(packet)) => {
                    info!("read packet success, {:?}", packet);
                }
                Ok(None) => {
                    info!("read packet failed, None");
                }
                Err(e) => {
                    info!("read packet failed, {:?}", e);
                    break;
                }
            }
        }

        Ok(())
    }
}
