use bytes::{Buf, BytesMut};
use tokio::io::{AsyncReadExt, BufWriter, AsyncWriteExt, AsyncSeekExt};
use tokio::net::TcpStream;

const MAX_BUFFER_SIZE: usize = 8192;

#[derive(Debug)]
pub struct Connection {
	stream: BufWriter<TcpStream>,
	buffer: BytesMut,
}

impl Connection {
	fn new(stream: TcpStream) -> Self {
		Self {
			stream: BufWriter::new(stream),
			buffer: BytesMut::with_capacity(MAX_BUFFER_SIZE),
		}
	}

	pub async fn read(&mut self) -> Result<(), Box<dyn std::error::Error>> {
		let mut buf = [0u8; 1024];
		let mut n = 0;
		loop {
			let r = self.stream.read(&mut buf).await?;
			if r == 0 {
				break;
			}
			n += r;
			self.buffer.extend_from_slice(&buf[..r]);
		}
		println!("read {} bytes", n);
		Ok(())
	}
}