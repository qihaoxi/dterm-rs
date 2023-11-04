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

struct TtyManager {
	server_addr: String,
	tty_count: i32,
	tty_map: std::collections::HashMap<String, Tty>,
	lock: tokio::sync::Mutex<()>,
	status: TtyStatus,
	sock:  tokio::net::TcpSocket,
}


impl TtyManager {
	fn new(addr: String) -> Self {

		Self {
			tty_count: 0,
			tty_map: std::collections::HashMap::new(),
			lock: tokio::sync::Mutex::new(()),
			status: TtyStatus::Disconnected,
			server_addr: addr,
			sock: tokio::net::TcpSocket::new_v4(),
		}
	}

	async fn connect(addr: String) -> Result<tokio::net::TcpStream, Box<dyn std::error::Error>> {
		let mut stream = TcpStream::connect(addr).await?;
		Ok(stream)
	}

	fn destroy(&mut self) {}

}