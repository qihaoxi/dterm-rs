use tokio::sync::mpsc::{self, channel, Receiver, Sender};
use tokio_util::sync::CancellationToken;

pub struct CancelCaller {
    token: CancellationToken,
    rx: Receiver<()>,
}

impl CancelCaller {
    pub fn cancel(&self) {
        self.token.cancel();
    }
    pub async fn wait(&mut self) {
        self.rx.recv().await;
    }

    pub async fn cancel_and_wait(&mut self) {
        self.token.cancel();
        self.wait().await;
    }
}

#[derive(Clone)]
pub struct CancelWatcher {
    token: CancellationToken,
    tx: Sender<()>,
}

impl CancelWatcher {
    pub async fn wait(&mut self) {
        self.token.cancelled().await;
    }
}

pub fn new_cancel() -> (CancelCaller, CancelWatcher) {
    let token = CancellationToken::new();
    let (tx, rx) = channel(1);

    let caller = CancelCaller {
        token: token.clone(),
        rx,
    };

    let watcher = CancelWatcher { token, tx };
    (caller, watcher)
}
