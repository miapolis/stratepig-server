use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::any::Any;
use std::io::Cursor;

use crate::buffer::NetworkBuffer;
use crate::Error;

pub const PACKET_HEADER_SIZE: usize = 3;
pub const MAX_PACKET_BODY_SIZE: usize = 8192;
pub const MAX_PACKET_SIZE: usize = PACKET_HEADER_SIZE + MAX_PACKET_BODY_SIZE;

#[derive(Clone)]
pub struct PacketHeader {
    pub size: u16,
    pub id: u8,
}

pub trait PacketBody: Any + Send + Sync {
    fn serialize(&self) -> Result<Vec<u8>, Error>;
    fn deserialize(_data: &[u8]) -> Result<Self, Error>
    where
        Self: Sized;
    fn id(&self) -> u8;
    fn box_clone(&self) -> Box<dyn PacketBody>;
}

impl Clone for Box<dyn PacketBody> {
    fn clone(&self) -> Box<dyn PacketBody> {
        self.box_clone()
    }
}

#[derive(Clone)]
pub struct Packet {
    pub header: PacketHeader,
    pub body: Vec<u8>,
}

pub fn serialize_packet(body: Box<dyn PacketBody>) -> Result<Vec<u8>, Error> {
    let mut body_data = body.serialize()?;

    let mut data: Vec<u8> = Vec::new();
    data.write_u16::<LittleEndian>(body_data.len() as u16)?;
    data.write_u8(body.id())?;

    data.append(&mut body_data);

    Ok(data)
}

pub fn deserialize_packet_header(buffer: &mut NetworkBuffer) -> Result<PacketHeader, Error> {
    let mut reader = Cursor::new(&buffer.data[..]);

    let body_size = reader.read_u16::<LittleEndian>()? as usize;

    if body_size >= MAX_PACKET_BODY_SIZE {
        eprintln!(
            "Packet body is {} bytes, max body size is {}",
            body_size, MAX_PACKET_BODY_SIZE
        );
        return Err(Error::InvalidData("packet body too large".to_owned()));
    }

    let packet_id = reader.read_u8()?;

    Ok(PacketHeader {
        size: body_size as u16,
        id: packet_id,
    })
}
