use bytes::{Buf, Bytes, BytesMut};
use std::any::Any;
use std::error;

use log::info;
use std::io::{self, Cursor, Seek};
use std::sync::{Arc, Mutex};
use tokio::io::{
    AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufReader, BufWriter, ReadHalf, SeekFrom, WriteHalf,
};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::OwnedMappedMutexGuard;
use tracing::debug;
// use tokio_util::bytes::Buf;

use crate::packet;
use crate::packet::Packet;

const MAX_BUFFER_SIZE: usize = 8192;

#[derive(Debug)]
pub struct Connection {
    // stream: TcpStream,
    // wr_stream: tokio::net::tcp::WriteHalf<'a>,
    // rd_stream: tokio::net::tcp::ReadHalf<'a>,
    // buffer: BytesMut,
    stream: Option<TcpStream>,
    wr_stream: Option<OwnedWriteHalf>,
    rd_stream: Option<OwnedReadHalf>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new() -> Self {
        Self {
            stream: None,
            wr_stream: None,
            rd_stream: None,
            buffer: BytesMut::with_capacity(MAX_BUFFER_SIZE),
        }
    }

    pub async fn connect(&mut self, addr: String) -> Result<(), Box<dyn std::error::Error>> {
        match TcpStream::connect(addr).await {
            Ok(stream) => {
                self.stream = Some(stream);
                let (rd_stream, wr_stream) = self.stream.take().unwrap().into_split();
                self.wr_stream = Some(wr_stream);
                self.rd_stream = Some(rd_stream);
                Ok(())
            }
            Err(e) => {
                info!("connect failed, {:?}", e);
                Err(Box::try_from(e).unwrap())
            }
        }
    }

    pub async fn destroy(&mut self) {
        match self.stream.take().unwrap().shutdown().await {
            Ok(_) => {
                info!("shutdown success");
            }
            Err(e) => {
                info!("shutdown failed, {:?}", e);
            }
        }
    }

    // register packet:
    // 1 byte packet type
    // 2 bytes packet length
    // 1 byte proto version
    // 4 bytes device id + '\0'
    // 4 bytes device name + '\0' optional
    // 4 bytes token + '\0' optional
    pub async fn register(
        &mut self,
        device_id: String,
        device_name: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let p = packet::Packet::new_register_packet(device_id, device_name);
        match self.write_packet(&p).await {
            Ok(_) => {
                info!("write packet success");
                Ok(())
            }
            Err(e) => {
                info!("write packet failed, {:?}", e);
                Err(Box::try_from(e).unwrap())
            }
        }
    }

    pub fn get_wr_stream(&mut self) -> &mut OwnedWriteHalf {
        self.wr_stream.as_mut().unwrap()
    }

    pub async fn write_packet(&mut self, packet: &Packet) -> io::Result<()> {
        self.wr_stream
            .as_mut()
            .unwrap()
            .write_u8(packet.packet_type.clone())
            .await?;
        self.wr_stream
            .as_mut()
            .unwrap()
            .write_u16(packet.packet_length.clone())
            .await?;
        self.wr_stream
            .as_mut()
            .unwrap()
            .write_all(&packet.packet_data)
            .await?;
        self.wr_stream.as_mut().unwrap().flush().await?;
        Ok(())
    }

    pub async fn read_packet(&mut self) -> Result<Option<Packet>, Box<dyn std::error::Error>> {
        // loop {
        //     if let Some(packet) = self.parse()? {
        //         info!("read packet success, {:?}", packet);
        //         return Ok(Some(packet));
        //     }
        //
        //     if 0 == self.rd_stream.as_mut().unwrap().read_buf(&mut self.buffer).await? {
        //         if self.buffer.is_empty() {
        // 			// no data to read, continue to next loop
        // 			debug!("read packet failed, None");
        //             continue;
        //         } else {
        //             info!("read packet failed, connection reset by peer");
        //             return Err("connection reset by peer".into());
        //         }
        //     }
        // }
        loop {
            // wait to read 3 bytes from stream, type(1 byte) + length(2 bytes)
            let mut buf = [0u8; 3];
            match self.rd_stream.as_mut().unwrap().read_exact(&mut buf).await {
                Ok(_) => {
                    let packet_type = buf[0];
                    let packet_length = u16::from_be_bytes([buf[1], buf[2]]);
                    let mut packet_data = BytesMut::with_capacity(packet_length as usize);
                    packet_data.resize(packet_length as usize, 0);

                    // wait to read packet_length bytes from stream
                    match self
                        .rd_stream
                        .as_mut()
                        .unwrap()
                        .read_exact(&mut packet_data)
                        .await
                    {
                        Ok(_) => {
                            let packet = packet::Packet::new(
                                packet_type,
                                packet_length,
                                packet_data.freeze(),
                            );
                            info!("read packet success, {:?}", packet);
                            return Ok(Some(packet));
                        }
                        Err(e) => {
                            info!("read packet failed, {:?}", e);
                            return Err(Box::try_from(e).unwrap());
                        }
                    }
                }
                Err(e) => {
                    info!("read packet failed, {:?}", e);
                    return Err(Box::try_from(e).unwrap());
                }
            }
        }
    }

    // pub async fn read_packet(&mut self) -> Result<Option<Packet>, Box<dyn std::error::Error>> {
    //     loop {
    //         if let Some(packet) = self.parse()? {
    //             info!("read packet success, {:?}", packet);
    //             return Ok(Some(packet));
    //         }
    //
    //         if 0 == self.rd_stream.read_buf(&mut self.buffer).await? {
    //             if self.buffer.is_empty() {
    //                 info!("read packet failed, None");
    //                 return Ok(None);
    //             } else {
    //                 info!("read packet failed, connection reset by peer");
    //                 return Err("connection reset by peer".into());
    //             }
    //         }
    //     }
    // }
    //
    // pub async fn write_packet(&mut self, packet: &Packet) -> io::Result<()> {
    //     self.wr_stream.write_u8(packet.packet_type.clone()).await?;
    //     self.wr_stream.write_u16(packet.packet_length.clone()).await?;
    //     self.wr_stream.write_all(&packet.packet_data).await?;
    //     self.wr_stream.flush().await?;
    //     Ok(())
    // }
    //
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
