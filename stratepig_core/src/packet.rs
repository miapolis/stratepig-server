use std::ops::Drop;
use std::str;

/// ## Packet
/// Packet struct used to contain buffer data for writing and reading data.
///
/// Contains no public fields; all data must be controlled through provided methods and functions.
/// ### Example
/// ```
/// // ...
/// use stratepig_core::Packet;
/// use stratepig_core::ServerMessage;
///
/// fn foo (mut stream: TcpStream) {
///     let mut packet = Packet::new_id(ServerMessage::Welcome as i32);
///     packet.write_str("0.0.1");
///     packet.write_str("123");
///     packet.write_length();
///     
///     stream.write(packet.to_array()).unwrap();
///     stream.flush().unwrap();    
/// }
/// ```
#[derive(Debug)]
pub struct Packet {
    buffer: Vec<u8>,
    readable_buffer: Option<Vec<u8>>,
    read_pos: usize,
}

impl Packet {
    /// Standard implementation of creating a new packet
    pub fn new() -> Packet {
        Packet {
            buffer: Vec::new(),
            read_pos: 0,
            readable_buffer: None,
        }
    }

    /// Creates a new packet from an ID that is used for sending
    pub fn new_id(id: i32) -> Packet {
        let mut packet = Packet::new();
        packet.write_i32(id);
        packet
    }

    /// Creates a new packet from provided bytes
    pub fn new_from_bytes(bytes: Vec<u8>) -> Packet {
        let mut packet = Packet::new();
        packet.set_bytes(&bytes[..]);
        packet
    }

    /// Sets the packet's buffer as the provided bytes
    pub fn set_bytes(&mut self, bytes: &[u8]) {
        self.buffer = bytes.to_vec();
    }

    /// Sets the readable buffer and returns the full buffer
    pub fn to_array(&mut self) -> &[u8] {
        self.readable_buffer = Some(self.buffer.clone());
        self.readable_buffer.as_ref().unwrap()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn unread_len(&self) -> usize {
        self.buffer.len() - self.read_pos
    }

    /// Writes the length of the packet buffer to the start of the buffer
    pub fn write_length(&mut self) {
        let arr = (self.buffer.len() as u32).to_le_bytes();
        self.buffer.splice(0..0, arr.iter().cloned());
    }

    /// Writes a single byte to the packet's buffer
    pub fn write_u8(&mut self, val: u8) {
        self.buffer.push(val);
    }

    /// Writes a single bool to the packet's buffer
    pub fn write_bool(&mut self, val: bool) {
        if val {
            self.buffer.push(1);
        } else {
            self.buffer.push(0);
        }
    }

    /// Writes an i32 to the packet's buffer
    pub fn write_i32(&mut self, val: i32) {
        self.buffer.extend(val.to_le_bytes().iter());
    }

    /// Writes an i64 to the packet's buffer
    pub fn write_i64(&mut self, val: i64) {
        self.buffer.extend(val.to_le_bytes().iter());
    }

    /// Writes a u32 to the packet's buffer
    pub fn write_u32(&mut self, val: u32) {
        self.buffer.extend(val.to_le_bytes().iter());
    }

    /// Writes a u64 to the packet's buffer
    pub fn write_u64(&mut self, val: u64) {
        self.buffer.extend(val.to_le_bytes().iter());
    }

    // Writes a string to the packet's buffer
    pub fn write_str(&mut self, val: &str) {
        self.write_i32(val.len() as i32);
        self.buffer.extend(val.as_bytes().iter());
    }

    /// Reads bytes from the packet's buffer
    pub fn read_u8s(&mut self, length: usize) -> Result<&[u8], &'static str> {
        if self.buffer.len() > self.read_pos {
            // Unread bytes
            let slice = &self.buffer[self.read_pos..self.read_pos + length];
            self.read_pos += length;
            Ok(slice)
        } else {
            Err("Could not read value of type '&[u8]'!")
        }
    }

    /// Reads bytes from the packet's buffer without moving the read pos
    pub fn read_u8s_no_move(&self, length: usize) -> Result<&[u8], &'static str> {
        if self.buffer.len() > self.read_pos {
            // Unread bytes
            let slice = &self.buffer[self.read_pos..self.read_pos + length];
            Ok(slice)
        } else {
            Err("Could not read value of type '&[u8]'!")
        }
    }

    /// Reads one i32 from the packet's buffer
    pub fn read_i32(&mut self) -> Result<i32, &'static str> {
        if self.buffer.len() > self.read_pos {
            let mut bytes: [u8; 4] = Default::default();
            bytes.copy_from_slice(&self.buffer[self.read_pos..self.read_pos + 4]);
            self.read_pos += 4;
            Ok(i32::from_le_bytes(bytes))
        } else {
            Err("Could not read value of type 'i32'!")
        }
    }

    /// Reads one i32 from the packet's buffer without moving the read pos
    pub fn read_i32_no_move(&self) -> Result<i32, &'static str> {
        if self.buffer.len() > self.read_pos {
            let mut bytes: [u8; 4] = Default::default();
            bytes.copy_from_slice(&self.buffer[self.read_pos..self.read_pos + 4]);
            Ok(i32::from_le_bytes(bytes))
        } else {
            Err("Could not read value of type 'i32'!")
        }
    }

    /// Reads one u32 from the packet's buffer
    pub fn read_u32(&mut self) -> Result<u32, &'static str> {
        if self.buffer.len() > self.read_pos {
            let mut bytes: [u8; 4] = Default::default();
            bytes.copy_from_slice(&self.buffer[self.read_pos..self.read_pos + 4]);
            self.read_pos += 4;
            Ok(u32::from_le_bytes(bytes))
        } else {
            Err("Could not read value of type 'u32'!")
        }
    }

    /// Reads one u32 from the packet's buffer without moving the read pos
    pub fn read_u32_no_move(&mut self) -> Result<u32, &'static str> {
        if self.buffer.len() > self.read_pos {
            let mut bytes: [u8; 4] = Default::default();
            bytes.copy_from_slice(&self.buffer[self.read_pos..self.read_pos + 4]);
            Ok(u32::from_le_bytes(bytes))
        } else {
            Err("Could not read value of type 'u32'!")
        }
    }

    /// Reads one f32 from the packet's buffer
    pub fn read_f32(&mut self) -> Result<f32, &'static str> {
        if self.buffer.len() > self.read_pos {
            let mut bytes: [u8; 4] = Default::default();
            bytes.copy_from_slice(&self.buffer[self.read_pos..self.read_pos + 4]);
            self.read_pos += 4;
            Ok(f32::from_le_bytes(bytes))
        } else {
            Err("Could not read value of type 'f32'!")
        }
    }

    /// Reads one f32 from the packet's buffer without moving the read pos
    pub fn read_f32_no_move(&self) -> Result<f32, &'static str> {
        if self.buffer.len() > self.read_pos {
            let mut bytes: [u8; 4] = Default::default();
            bytes.copy_from_slice(&self.buffer[self.read_pos..self.read_pos + 4]);
            Ok(f32::from_le_bytes(bytes))
        } else {
            Err("Could not read value of type 'f32'!")
        }
    }

    /// Reads one bool from the packet's buffer
    pub fn read_bool(&mut self) -> Result<bool, &'static str> {
        if self.buffer.len() > self.read_pos {
            let byte = self.buffer[self.read_pos];
            self.read_pos += 1;
            match byte {
                1 => Ok(true),
                _ => Ok(false),
            }
        } else {
            Err("Could not read value of type 'bool'!")
        }
    }

    /// Reads one bool from the packet's buffer without moving the read pos
    pub fn read_bool_no_move(&self) -> Result<bool, &'static str> {
        if self.buffer.len() > self.read_pos {
            let byte = self.buffer[self.read_pos];
            match byte {
                1 => Ok(true),
                _ => Ok(false),
            }
        } else {
            Err("Could not read value of type 'bool'!")
        }
    }

    /// Reads a string from the packet's buffer
    pub fn read_string(&mut self) -> Result<String, &'static str> {
        if self.buffer.len() > self.read_pos {
            let length = match self.read_i32() {
                Ok(length) => length,
                Err(_) => return Err("1 Could not read value of type 'string'!"),
            };
            if length <= 0 {
                return Err("2 Could not read value of type 'string'");
            }
            let contents = String::from_utf8(
                self.buffer[self.read_pos..self.read_pos + length as usize].to_vec(),
            );
            self.read_pos += length as usize;

            match contents {
                Ok(string) => return Ok(string),
                Err(_) => return Err("3 Could not read value of type 'string'!"),
            }
        } else {
            Err("4 Could not read value of type 'string'!")
        }
    }

    /// Reads a string from the packet's buffer without moving the read pos
    pub fn read_string_no_move(&mut self) -> Result<String, &'static str> {
        if self.buffer.len() > self.read_pos {
            let length = match self.read_i32() {
                Ok(length) => length,
                Err(_) => return Err("Could not read value of type 'string'!"),
            };
            if length <= 0 {
                return Err("Could not read value of type 'string'");
            }
            let contents = String::from_utf8(
                self.buffer[self.read_pos..self.read_pos + length as usize].to_vec(),
            );

            match contents {
                Ok(string) => return Ok(string),
                Err(_) => return Err("Could not read value of type 'string'!"),
            }
        } else {
            Err("Could not read value of type 'string'!")
        }
    }
}

impl Drop for Packet {
    fn drop(&mut self) {
        self.buffer.clear();
        self.readable_buffer = None;
        self.read_pos = 0;
    }
}
