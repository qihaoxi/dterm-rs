use std::any::Any;
use std::error;
use bytes::{Buf, BytesMut, Bytes};

use tokio::io::{AsyncReadExt, BufWriter, AsyncWriteExt, AsyncSeekExt};
use tokio::net::TcpStream;
use std::io::{self, Cursor, Seek};
// use tokio_util::bytes::Buf;

use crate::packet;
use crate::packet::Packet;


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

	pub async fn read(&mut self) -> Result<Option<Packet>, Box<dyn std::error::Error>> {
		loop {
			if let Some(packet) = self.parse()? {
				return Ok(Some(packet));
			}

			if 0 == self.stream.read_buf(&mut self.buffer).await? {
				if self.buffer.is_empty() {
					return Ok(None);
				} else {
					return Err("connection reset by peer".into());
				}
			}
		}
	}

	pub async fn write(&mut self, packet: &Packet) -> io::Result<> {

	}

	fn parse(&mut self) -> std::result::Result<Option<Packet>, Box<dyn error::Error>> {
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
