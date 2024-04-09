use std::collections::HashMap;
use crate::connections::Connection;
use crate::packet;
use log::{debug, info};
use log4rs;
use std::future::Future;
use std::{io, thread, time};
use tokio::{io::{AsyncBufReadExt, AsyncWriteExt}, join, net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpListener, TcpStream,
}, sync::{mpsc, Mutex}, task};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, BufReader, BufWriter};
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::error;
use crate::cancel::{CancelCaller, CancelWatcher};
use crate::myerror::MyError;
use crate::packet::Packet;

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

    lock: tokio::sync::Mutex<()>,
    status: TtyStatus,

    cancel_watcher: CancelWatcher,// cancel watcher  for exit,
}

impl TtyManager {
    pub(crate) fn new(addr: String, cancel_watcher: CancelWatcher) -> Self {
        Self {
            server_addr: addr,
            // tty_count: 0,
            // tty_map: std::collections::HashMap::new(),
            lock: tokio::sync::Mutex::new(()),
            status: TtyStatus::Disconnected,
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

    pub async fn main_loop(&mut self) -> Result<(), std::io::Error> {
        /*
            1. connect to server
            2. send register packet to server and read response
            3. create loop read/write net in thread.
               read from server, write to channel;
               read from channel, write to server
        */

        let mut inactive = 0;
        info!("start connect: {}", self.server_addr);
        let mut stream = match TcpStream::connect(self.server_addr.clone()).await {
            Ok(s) => {
                info!("connect success");
                s
            }
            Err(e) => {
                info!("connect failed, {:?}", e);
                return Err(e);
            }
        };


        //write "dterm" to server
        /*match stream.write_all(b"dterm:").await {
            Ok(_) => {
                info!("write dterm success");
            }
            Err(e) => {
                info!("write dterm failed, {:?}", e);
                return Err(e);
            }``````
        }*/

        let (net_reader, net_writer) = stream.into_split();
        let mut reader = tokio::io::BufReader::new(net_reader);
        let mut writer = tokio::io::BufWriter::new(net_writer);

        // hashmap to store user's tty and its status
        let mut tty_map: HashMap<&str, i32> = std::collections::HashMap::new();

        //create channel for network read, dispatch thread to process packet
        let (nr_tx, mut nr_rx) = mpsc::channel::<Packet>(1024);

        // create channel for network write, read from channel, write to server
        let (nw_tx, nw_rx) = mpsc::channel::<Packet>(1024);

        // create and write register packet
        info!("create register packet");
        let register_packet = packet::Packet::new_register_packet("127-0-0-1".to_string(), "127".to_string());


        /*// create read thread to read from server, write to channel
        let read_task = thread::scope(|s| {
            info!("spawn write net");
            // create read thread to read from server, write to channel
            // let mut read_task = self.read_net(reader, nr_tx).await;
            let r = self.read_net(reader, nr_tx);
        });


        let write_task = thread::scope(|s| {
            info!("spawn write net");
            // create write thread to read from channel, write to server
            // let mut write_task = self.write_net(writer, nw_rx).await;
            let w = self.write_net(writer, nw_rx);
        });

        // create dispatch thread to process packet from recv channel
        let dispatch_task = thread::scope(|s| {
            info!("spawn dispatch packet");
            // create dispatch thread to process packet from recv channel
            let d = self.dispatch_packet(&mut nr_rx, &mut tty_map);
        });*/

        let r = self.read_net(reader, nr_tx);

        let self2 = Arc::new(self.clone);
        let w = self.write_net(writer, nw_rx);
        let d = self.dispatch_packet(&mut nr_rx, &mut tty_map);


        // send register packet to send channel
        // nw_tx.send(register_packet).await;


        // wait for all tasks to finish
        info!("wait for all tasks to finish");
        self.cancel_watcher.wait().await;
        Ok(())
    }

    async fn read_net(&mut self, mut reader: BufReader<OwnedReadHalf>, nr_tx: Sender<Packet>) {
        info!("start read net");
        loop {
            // read 3 bytes from stream, type(1 byte) + length(2 bytes)
            let mut header = [0u8; 3];
            match reader.read_exact(&mut header).await {
                Ok(n) => {
                    let packet_type = header[0];
                    let packet_length = u16::from_be_bytes([header[1], header[2]]);
                    let mut packet_data = bytes::BytesMut::with_capacity(packet_length as usize);
                    packet_data.resize(packet_length as usize, 0);

                    // read packet_length bytes from stream
                    reader.read_exact(&mut packet_data).await.unwrap();
                    info!("read packet success, type:{}, length:{}, data: {}", packet_type, packet_length, String::from_utf8(packet_data.to_vec()).unwrap());

                    let packet = packet::Packet::new(packet_type, packet_length, packet_data.freeze());
                    // send to channel, dispatch thread to process
                    nr_tx.send(packet).await.unwrap();
                }
                Err(e) => {
                    info!("read packet failed, {:?}", e);
                    break;
                }
            }
        }
    }

    async fn write_net(&mut self, mut writer: BufWriter<OwnedWriteHalf>, mut nw_rx: Receiver<Packet>) {
        info!("start write net");
        loop {
            // read from channel
            match nw_rx.recv().await {
                Some(packet) => {
                    info!("recv msg from channel");
                    match self.write_packet(&mut writer, &packet).await {
                        Ok(_) => {
                            info!("write packet success");
                        }
                        Err(e) => {
                            info!("write packet failed, {:?}", e);
                            break;
                        }
                    }
                }
                None => {
                    info!("channel closed, exit");
                    break;
                }
            }
        }
    }

    async fn dispatch_packet(&mut self, nr_rx: &mut Receiver<Packet>, tty_map: &mut HashMap<&str, i32>) {
        info!("start dispatch packet");
        loop {
            match nr_rx.recv().await {
                Some(packet) => {
                    info!("recv packet from channel");
                    self.handle_packet(packet).await;
                }
                None => {
                    info!("channel closed, exit");
                    break;
                }
            }
        }
    }

    async fn write_packet(&mut self, stream: &mut BufWriter<OwnedWriteHalf>, packet: &Packet) -> io::Result<()> {
        let data = packet.to_bytes();
        stream.write_all(&data).await?;
        stream.flush().await?;
        Ok(())
    }

    async fn handle_packet(&mut self, packet: Packet) {
        let packet_type = packet.packet_type as u8;
        match packet_type {
            1 => {
                //login
                info!("recv login packet");
            }
            2 => {
                //logout
                info!("recv logout packet");
            }
            3 => {
                //termdata
                info!("recv termdata packet");
            }
            4 => {
                //winsize
                info!("recv winsize packet");
            }
            9 => {
                //ack
                info!("recv ack packet");
            }

            // 5 6 7 8 are ignored temporarily
            5 => {
                //cmd
                info!("recv cmd packet");
            }
            6 => {
                //heartbeat
                info!("recv heartbeat packet");
            }
            7 => {
                //file
                info!("recv file packet");
            }
            8 => {
                //http
                info!("recv http packet");
            }

            _ => {
                info!("recv unknown packet");
            }
        }
    }
}

async fn test_tty_manager1() {
    let addr = "";
    println!("start test_tty_manager");
}

#[tokio::test]
async fn test_tty_manager() {
    thread::scope(|scope| {
        test_tty_manager1()
    }).await;


    // create a scope
    // thread::scope(|scope| {
    //
    //     // spawn first thread
    //     let r1 = scope.spawn(|| async {
    //         test_tty_manager1().await;
    //     });
    //
    //     // spawn second thread
    //     scope.spawn(|| {
    //         thread::sleep(time::Duration::from_secs(2));
    //         // wait for 2 seconds before printing "Hello, from thread 2"
    //         println!("Hello, from thread 2");
    //     });
    //
    //     // spawn third thread
    //     scope.spawn(|| {
    //         thread::sleep(time::Duration::from_secs(1));
    //         // wait for 10 seconds before printing "Hello, from thread 3"
    //         println!("Hello, from thread 3");
    //     });
    // });

    // all threads within the scope has to be closed
    // for the program to continue
    thread::sleep(time::Duration::from_secs(5));
    println!("All threads completed!");
}