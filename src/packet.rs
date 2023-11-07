use bytes::{Buf, Bytes};
use std::io::Cursor;

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

#[derive(Clone, Debug)]
pub struct Packet {
    pub packet_type: u8,
    pub packet_length: u16,
    pub packet_data: bytes::Bytes,
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
