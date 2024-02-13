use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::io::{Cursor, Write};
use std::ptr::write;
use tracing::callsite::register;
// use bytebuffer::ByteBuffer;
use crate::packet::PacketType::Register;

#[derive(Clone, Debug)]
pub enum PacketType {
    Register(u8),
    Login(u8),
    Logout(u8),
    Termdata(u8),
    Winsize(u8),
    Cmd(u8),
    Heartbeat(u8),
    File(u8),
    Http(u8),
    Ack(u8),
    PktTypeMax(u8),
}

impl std::convert::From<PacketType> for u8 {
    fn from(packet_type: PacketType) -> Self {
        match packet_type {
            PacketType::Register(_) => 0,
            PacketType::Login(_) => 1,
            PacketType::Logout(_) => 2,
            PacketType::Termdata(_) => 3,
            PacketType::Winsize(_) => 4,
            PacketType::Cmd(_) => 5,
            PacketType::Heartbeat(_) => 6,
            PacketType::File(_) => 7,
            PacketType::Http(_) => 8,
            PacketType::Ack(_) => 9,
            PacketType::PktTypeMax(_) => 9,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Packet {
    pub packet_type: u8,
    pub packet_length: u16,
    pub packet_data: bytes::Bytes,
}

impl Packet {
    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(3 + self.packet_data.len());
        buf.put_u8(self.packet_type);
        buf.put_u16(self.packet_length);
        buf.put(self.packet_data.clone());
        buf.freeze().to_vec()
    }
}

#[derive(Debug)]
pub enum Error {
    /// Not enough data is available to parse a message
    Incomplete,

    /// Invalid message encoding
    Other(Box<dyn std::error::Error + Send + Sync>),
}

impl Packet {
    pub fn new(packet_type: u8, packet_length: u16, packet_data: Bytes) -> Self {
        Self {
            packet_type,
            packet_length,
            packet_data,
        }
    }

    /// type/len(2 bytes)/proto/device_id/0/desc/0/token/0
    pub fn new_register_packet(device_id: String, desc: String) -> Self {
        let len = 4 + device_id.len() + desc.len();
        let mut buf = BytesMut::new();

        buf.put_u8(3); //proto version:3
        buf.put_slice(device_id.as_bytes()); //device_id
        buf.put_u8(b'0'); //device_id end
        buf.put_slice(desc.as_bytes()); //desc
        buf.put_u8(b'0'); //desc end
        buf.put_u8(b'0'); //token end

        Self {
            packet_type: PacketType::Register(0).into(),
            packet_length: len as u16,
            packet_data: buf.freeze(),
        }
    }

    pub fn new_login_packet() -> Self {
        Self {
            packet_type: PacketType::Login(0).into(),
            packet_length: 0,
            packet_data: Bytes::new(),
        }
    }

    pub fn from_buffer(buffer: &mut Cursor<&[u8]>) -> Result<Self, Error> {
        let packet_type = get_u8(buffer)?;
        let packet_length = get_u16(buffer)?;
        let packet_data = buffer.copy_to_bytes(packet_length as usize);
        Ok(Self {
            packet_type,
            packet_length,
            packet_data,
        })
    }

    pub fn to_buffer(
        &mut self,
        buffer: &mut Cursor<&mut [u8]>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        buffer.write(self.packet_type.to_be_bytes().as_ref())?;
        buffer.write(self.packet_length.to_be_bytes().as_ref())?;
        buffer.write(self.packet_data.as_ref())?;
        Ok(())
    }
}

pub(crate) fn get_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }

    Ok(src.get_u8())
}

pub(crate) fn get_u16(src: &mut Cursor<&[u8]>) -> Result<u16, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }

    Ok(src.get_u16())
}

fn peek_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }

    Ok(src.chunk()[0])
}
