use crate::connections::Connection;
use crate::packet;
use log::info;
use log4rs;
use std::future::Future;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use crate::cancel::{CancelCaller, CancelWatcher};

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
    cancel_watcher: CancelWatcher,// cancel watcher  for exit,
}

impl TtyManager {
    pub(crate) fn new(addr: String, cancel_watcher: CancelWatcher) -> Self {
        Self {
            tty_count: 0,
            tty_map: std::collections::HashMap::new(),
            lock: tokio::sync::Mutex::new(()),
            status: TtyStatus::Disconnected,
            server_addr: addr,
            cancel_watcher,
        }
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

    // connect to server;register to server;read response from server; read loop & write loop
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("start connect: {}", self.server_addr);

        let mut connection = Connection::new();
        match connection.connect(self.server_addr.clone()).await {
            Ok(_) => {
                info!("connect success");
            }
            Err(e) => {
                info!("connect failed, {:?}", e);
                return Err(Box::try_from(e).unwrap());
            }
        }

        // write register packet
        let p = packet::Packet::new_register_packet(
            String::from("172-30-97-139"),
            String::from("172-30-97-139"),
        );
        match connection.write_packet(&p).await {
            Ok(_) => {
                info!("write packet success");
            }
            Err(e) => {
                info!("write packet failed, {:?}", e);
                return Err(Box::try_from(e).unwrap());
            }
        }
        info!("write register packet");

        // read response packet
        match connection.read_packet().await {
            Ok(Some(packet)) => {
                info!("read packet success, {:?}", packet);
            }
            Ok(None) => {
                info!("read packet failed, None");
            }
            Err(e) => {
                info!("read packet failed, {:?}", e);
                return Err(Box::try_from(e).unwrap());
            }
        }
        info!("read register response packet");


        // read loop
        let r = tokio::spawn(async move {
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
        });

        // let w = tokio::spawn(async move {
        //     loop {
        //         match connection.write_packet(&p).await {
        //             Ok(_) => {
        //                 info!("write packet success");
        //             }
        //             Err(e) => {
        //                 info!("write packet failed, {:?}", e);
        //                 break;
        //             }
        //         }
        //     }
        // });

        //write loop
        match tokio::join!(r) {
            (Ok(_), ) => {
                info!("tty_manager exit success");
            }
            (Err(e), ) => {
                info!("tty_manager exit failed, err: {:?}", e);
            }
        }
        Ok(())
    }
}
