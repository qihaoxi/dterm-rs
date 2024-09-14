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