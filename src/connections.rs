use std::any::Any;
use bytes::{Buf, BytesMut, Bytes};

use tokio::io::{AsyncReadExt, BufWriter, AsyncWriteExt, AsyncSeekExt};
use tokio::net::TcpStream;
use std::io::{self, Cursor, Seek};

use crate::packet;


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


	fn parse(&mut self) -> std::result::Result<Option<packet::Packet>, Box<dyn std::error::Error + Send + Sync>> {
		use packet::Error::Incomplete;
		let mut buf = Cursor::new(&self.buffer[..]);

		if buf.remaining() < 3 {
			return Ok(None);
		}

		let packet_type = buf.get_u8();
		let packet_length = buf.get_u16();
		buf.advance(3);
		if buf.remaining() < packet_length.clone() as usize {
			return Ok(None);
		}

		let packet_data = buf.copy_to_bytes(packet_length.clone() as usize);
		let packet = packet::Packet::new(packet_type, packet_length, packet_data);
		Ok(Some(packet))
	}
}