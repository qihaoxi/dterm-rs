use std::collections::HashMap;
use crate::connections::Connection;
use crate::packet;
use log::{debug, info};
use log4rs;
use std::future::Future;
use std::io;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener, TcpStream,
    },
    sync::{mpsc, Mutex},
};
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
    // tty_count: i32,
    // tty_map: std::collections::HashMap<String, Tty>,
    lock: tokio::sync::Mutex<()>,
    status: TtyStatus,

    // receive packet channel for network
    //  recv_chan: tokio::sync::mpsc::Receiver<()>,
    // send packet channel for network
    // send_chan: tokio::sync::mpsc::Sender<()>,

    // inactive: i32,
    // active: tokio::time::Duration,
    // last_heartbeat: tokio::time::Duration, //timestamp

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


    // connect to server;register to server;read response from server; read loop & write loop
    // pub async fn run(&mut self) -> Result<(), ()> {
    //     info!("start connect: {}", self.server_addr);
    //     let connection = Arc::new(Mutex::new(Connection::new()));
    //     match connection.lock().await.connect(self.server_addr.clone()).await {
    //         Ok(_) => {
    //             info!("connect success");
    //         }
    //         Err(e) => {
    //             info!("connect failed, {:?}", e);
    //             // convert e to MyError
    //             //impl send for MyError
    //             return Err(());
    //         }
    //     }
    //
    //     //write "dterm" to server
    //     let mut s = connection.lock().await;
    //     match s.get_wr_stream().write_all(b"dterm").await {
    //         Ok(_) => {
    //             info!("write dterm success");
    //         }
    //         Err(e) => {
    //             info!("write dterm failed, {:?}", e);
    //             return Err(());
    //         }
    //     }
    //
    //     // send login packet
    //     let login_packet = packet::Packet::new_login_packet();
    //     match connection.lock().await.write_packet(&login_packet).await {
    //         Ok(_) => {
    //             info!("write login packet success");
    //         }
    //         Err(e) => {
    //             info!("write login packet failed, {:?}", e);
    //             return Err(());
    //         }
    //     }
    //
    //
    //     // channel for network read
    //     let (nr_tx, mut nr_rx) = tokio::sync::mpsc::channel(1024);
    //     // channel for network write
    //     let (nw_tx, mut nw_rx) = tokio::sync::mpsc::channel(1024);
    //
    //
    //     // socket read loop, read from server, write to channel
    //     let connection_read = Arc::clone(&connection);
    //     let mut r = async {
    //         loop {
    //             let mut buf = [0u8; 8192];
    //             match connection_read.lock().await.read_packet().await {
    //                 Ok(packet) => {
    //                     // send to channel, dispatch thread to process
    //                     nr_tx.send(packet).await.unwrap();
    //                 }
    //                 Err(e) => {
    //                     info!("read packet failed, {:?}", e);
    //                     break;
    //                 }
    //             }
    //         }
    //     };
    //
    //     // dispatch thread to process packet
    //     let mut d = async {
    //         loop {
    //             match nr_rx.recv().await {
    //                 Some(packet) => {
    //                     info!("recv msg from channel");
    //                     // process packet
    //                 }
    //                 None => {
    //                     info!("channel closed, exit");
    //                     break;
    //                 }
    //             }
    //         }
    //     };
    //
    //
    //     // read from channel, write to server
    //     // let mut w = async {
    //     //     loop {
    //     //         // read from channel
    //     //         match nw_rx.recv().await {
    //     //             Some(packet) => {
    //     //                 info!("recv msg from channel");
    //     //                 // write to server
    //     //                 match connection.write_packet(&packet).await {
    //     //                     Ok(_) => {
    //     //                         info!("write packet success");
    //     //                     }
    //     //                     Err(e) => {
    //     //                         info!("write packet failed, {:?}", e);
    //     //                         break;
    //     //                     }
    //     //                 }
    //     //             }
    //     //             None => {
    //     //                 info!("channel closed, exit");
    //     //                 break;
    //     //             }
    //     //         }
    //     //     }
    //     // };
    //
    //     let connection_write = Arc::clone(&connection);
    //     let mut w = async {
    //         loop {
    //             // read from channel
    //             match nw_rx.recv().await {
    //                 Some(packet) => {
    //                     info!("recv msg from channel");
    //                     // write to server
    //                     match connection_write.lock().await.write_packet(&packet).await {
    //                         Ok(_) => {
    //                             info!("write packet success");
    //                         }
    //                         Err(e) => {
    //                             info!("write packet failed, {:?}", e);
    //                             break;
    //                         }
    //                     }
    //                 }
    //                 None => {
    //                     info!("channel closed, exit");
    //                     break;
    //                 }
    //             }
    //         }
    //     };
    //
    //
    //     // wait for exit loop
    //     let mut e = async {
    //         self.cancel_watcher.wait().await;
    //         info!("cancel_watcher cancelled");
    //     };
    //
    //     // wait for all tasks to finish
    //     select! {
    //         _ = r => {
    //             info!("r finished");
    //         }
    //         _ = d => {
    //             info!("d finished");
    //         }
    //         _ = w => {
    //             info!("w finished");
    //         }
    //         _ = e => {
    //             info!("e finished");
    //         }
    //     }
    //
    //     Ok(())
    // }

    pub async fn main_loop(&mut self) -> Result<(), std::io::Error> {
        // connect to server;register to server;read response from server; read loop & write loop
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
        let mut inactive = 0;


        //write "dterm" to server
        /*match stream.write_all(b"dterm:").await {
            Ok(_) => {
                info!("write dterm success");
            }
            Err(e) => {
                info!("write dterm failed, {:?}", e);
                return Err(e);
            }
        }*/

        let (net_reader, net_writer) = stream.into_split();
        let mut reader = tokio::io::BufReader::new(net_reader);
        let mut writer = tokio::io::BufWriter::new(net_writer);


        // match TtyManager::write_packet(&mut writer, &register_packet).await {
        //     Ok(_) => {
        //         info!("write register packet success");
        //     }
        //     Err(e) => {
        //         info!("write register packet failed, {:?}", e);
        //         return Err(e);
        //     }
        // }

        // create channel for network read, dispatch thread to process packet
        let (nr_tx, mut nr_rx) = mpsc::channel::<Packet>(1024);
        let mut read_task = tokio::spawn(async move {
            // TtyManager::read_net(reader, nr_tx).await;
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
                        inactive = 0;
                    }
                    Err(e) => {
                        info!("read packet failed, {:?}", e);
                        inactive = 0;
                        break;
                    }
                }
            }
        });

        let mut tty_map: HashMap<&str, i32> = std::collections::HashMap::new();

        // create dispatch thread to process packet
        let mut dispatch_task = tokio::spawn(async move {
            loop {
                match nr_rx.recv().await {
                    Some(packet) => {
                        info!("recv packet from channel");
                        // process packet
                        let packet_type = packet.packet_type as u8;
                        match packet_type {
                            0 => {
                                //register
                                info!("recv register packet");
                                // first byte of packet_data is 0, indicating register success
                                if packet.packet_data[0] == 0 {
                                    info!("register success");
                                } else {
                                    info!("register failed");
                                    return;
                                }
                            }
                            1=> {
                                //login
                                info!("recv login packet");
                                TtyManager::handle_packet(packet).await;
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
                    None => {
                        info!("channel closed, exit");
                        break;
                    }
                }
            }
        });

        // create channel for network write, read from channel, write to server
        let (nw_tx, nw_rx) = mpsc::channel::<Packet>(1024);
        let write_task = tokio::spawn(async move {
            TtyManager::write_net(writer, nw_rx).await;
        });

        // wait for exit loop
        let mut cancel_watcher_clone = self.cancel_watcher.clone(); // assuming `self.cancel_watcher` is cloneable
        let exit_task = tokio::spawn(async move {
            cancel_watcher_clone.wait().await;
            info!("cancel_watcher cancelled");
        });

        // write register packet
        let register_packet = packet::Packet::new_register_packet("127-0-0-1".to_string(), "127".to_string());
        nw_tx.send(register_packet).await.unwrap();

        // wait for all tasks to finish
        let _ = tokio::try_join!(read_task, write_task, exit_task);
        Ok(())
    }

    async fn read_net(mut reader: BufReader<OwnedReadHalf>, nr_tx: Sender<Packet>) {
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

    async fn write_net(mut writer: BufWriter<OwnedWriteHalf>, mut nw_rx: Receiver<Packet>) {
        loop {
            // read from channel
            match nw_rx.recv().await {
                Some(packet) => {
                    info!("recv msg from channel");
                    match TtyManager::write_packet(&mut writer, &packet).await {
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
    pub async fn write_packet(stream: &mut BufWriter<OwnedWriteHalf>, packet: &Packet) -> io::Result<()> {
        let data = packet.to_bytes();
        stream.write_all(&data).await?;
        stream.flush().await?;
        Ok(())
    }

    async fn handle_packet(packet: Packet) {
        let packet_type = packet.packet_type as u8;
        match packet_type {
            1=> {
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
