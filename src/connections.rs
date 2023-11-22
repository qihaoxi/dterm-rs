use bytes::{Buf, Bytes, BytesMut};
use std::any::Any;
use std::error;

use log::info;
use std::io::{self, Cursor, Seek};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufReader, BufWriter, SeekFrom};
use tokio::net::TcpStream;
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
    pub(crate) fn new(stream: TcpStream) -> Self {
        Self {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(MAX_BUFFER_SIZE),
        }
    }

    pub async fn read_packet(&mut self) -> Result<Option<Packet>, Box<dyn std::error::Error>> {
        loop {
            if let Some(packet) = self.parse()? {
                info!("read packet success, {:?}", packet);
                return Ok(Some(packet));
            }

            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                if self.buffer.is_empty() {
                    info!("read packet failed, None");
                    return Ok(None);
                } else {
                    info!("read packet failed, connection reset by peer");
                    return Err("connection reset by peer".into());
                }
            }
        }
    }

    pub async fn write_packet(&mut self, packet: &Packet) -> io::Result<()> {
        self.stream.write_u8(packet.packet_type.clone()).await?;
        self.stream.write_u16(packet.packet_length.clone()).await?;
        self.stream.write_all(&packet.packet_data).await?;
        self.stream.flush().await?;
        Ok(())
    }

    fn parse(&mut self) -> std::result::Result<Option<Packet>, Box<dyn error::Error>> {
        use packet::Error::Incomplete;
        let mut buf = Cursor::new(&self.buffer[..]);

        let size = buf.remaining();
        info!("parse buffer size: {}", size);
        if size < 3 {
            info!("parse failed, {:?}", Incomplete);
            return Ok(None);
        }

        let packet_type = buf.get_u8();
        let packet_length = buf.get_u16();
        if buf.remaining() < packet_length.clone() as usize {
            return Ok(None);
        }

        let packet_data = buf.copy_to_bytes(packet_length.clone() as usize);
        let packet = packet::Packet::new(packet_type, packet_length, packet_data);
        Ok(Some(packet))
    }
}
