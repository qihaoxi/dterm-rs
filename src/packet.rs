use std::io::Cursor;
use bytes::{Buf, Bytes};


enum PacketType  {
	PktTypeRegister,
	PktTypeLogin,
	PktTypeLogout,
	PktTypeTermdata,
	PktTypeWinsize,
	PktTypeCmd,
	PktTypeHeartbeat,
	PktTypeFile,
	PktTypeHttp,
	PktTypeAck,
	PktTypeMax,
}

#[derive(Clone, Debug)]
pub struct Packet{
	pub packet_type: u8,
	pub packet_length: u16,
	pub packet_data: Vec<u8>,
}

#[derive(Debug)]
pub enum Error {
	/// Not enough data is available to parse a message
	Incomplete,

	/// Invalid message encoding
	Other(Box<dyn std::error::Error + Send + Sync>),
}

impl Packet {
	pub fn new(packet_type: u8, packet_length: u16, packet_data: Vec<u8>) -> Self {
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

