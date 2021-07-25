//! ## Core
//! The core module for Stratepig containing server, client, packet data and more.
//! ### Modules
//! - Packet (contains packet implementation for writing and reading data as bytes)
//! - Server (contains function to start the server and listen at a port)
//!
//! ### Example
//! #### How to start a server
//! ```
//! use stratepig_core::server;
//!
//! fn main() {
//!     server::start();
//! }
//! ```
use mio::net::TcpStream;
pub use mio::Token;
use std::io::Write;

mod buffer;
mod error;
mod packet;

pub use error::Error;
pub use packet::*;
pub mod server;

pub enum PacketRecipient {
    All,
    Single(Token),
    Exclude(Token),
    ExcludeMany(Vec<Token>),
    Include(Vec<Token>),
}

pub fn send_bytes(socket: &mut TcpStream, buffer: &[u8]) -> Result<usize, Error> {
    let mut len = buffer.len();
    if len == 0 {
        return Err(Error::InvalidData("attempting to send 0 bytes".to_owned()));
    }

    // Keep sending until we've sent the entire buffer
    while len > 0 {
        match socket.write(buffer) {
            Ok(sent_bytes) => {
                len -= sent_bytes;
            }
            Err(_) => {
                return Err(Error::FailedToSendBytes);
            }
        }
    }

    Ok(buffer.len())
}
