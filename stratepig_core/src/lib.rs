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
mod packet;

pub use packet::*;
pub mod server;
