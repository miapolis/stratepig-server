use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use std::collections::{HashMap, VecDeque};
use std::io::Read;
use std::net::SocketAddr;

use crate::buffer::NetworkBuffer;
use crate::error::Error;
use crate::packet::{
    deserialize_packet_header, serialize_packet, Packet, PacketBody, PACKET_HEADER_SIZE,
};
use crate::send_bytes;
use crate::PacketRecipient;

const LOCAL_TOKEN: Token = Token(0);
const EVENTS_CAPACITY: usize = 4096;

#[derive(Debug)]
pub enum ServerEvent {
    ClientConnected(Token, SocketAddr),
    ClientDisconnected(Token),
    ReceivedPacket(Token, usize),
    SentPacket(Token, usize),

    #[doc(hidden)]
    __Nonexhaustive,
}

pub struct Connection {
    token: Token,
    socket: TcpStream,
    disconnected: bool,
    buffer: NetworkBuffer,
    outgoing_packets: VecDeque<Box<dyn PacketBody>>,
}

impl Connection {
    pub fn new(token: Token, socket: TcpStream) -> Self {
        Connection {
            token,
            socket,
            disconnected: false,
            buffer: NetworkBuffer::new(),
            outgoing_packets: VecDeque::new(),
        }
    }
}

pub struct Server {
    tcp_listener: TcpListener,
    events: Events,
    poll: Poll,
    ms_sleep_time: u16,
    connections: HashMap<Token, Connection>,
    token_counter: usize,
    incoming_packets: VecDeque<(Token, Packet)>,
}

impl Server {
    pub fn new(ip: &str, port: u16, tickrate: u16) -> Result<Server, Error> {
        let addr = format!("{}:{}", ip, port).parse().unwrap();
        let mut tcp_listener = TcpListener::bind(addr)?;

        let poll = Poll::new().unwrap();
        poll.registry()
            .register(&mut tcp_listener, LOCAL_TOKEN, Interest::READABLE)?;

        Ok(Server {
            tcp_listener,
            events: Events::with_capacity(EVENTS_CAPACITY),
            poll,
            ms_sleep_time: 1000 / tickrate,
            connections: HashMap::new(),
            token_counter: 0,
            incoming_packets: VecDeque::new(),
        })
    }

    pub fn num_connections(&self) -> usize {
        self.connections.len()
    }

    pub fn drain_incoming_packets(&mut self) -> Vec<(Token, Packet)> {
        self.incoming_packets.drain(..).collect()
    }

    pub fn kick(&mut self, connection_token: Token) -> Result<(), Error> {
        let conn: &mut Connection = match self.connections.get_mut(&connection_token) {
            Some(c) => c,
            None => return Err(Error::ConnectionNotFound),
        };

        conn.disconnected = true;

        Ok(())
    }

    pub fn send(&mut self, recipient: PacketRecipient, packet: impl PacketBody) {
        let boxed: Box<dyn PacketBody> = Box::new(packet);
        self.send_boxed(recipient, boxed);
    }

    pub fn send_boxed(&mut self, recipient: PacketRecipient, packet_boxed: Box<dyn PacketBody>) {
        match recipient {
            PacketRecipient::All => {
                for (_, connection) in self.connections.iter_mut() {
                    connection.outgoing_packets.push_back(packet_boxed.clone());
                }
            }
            PacketRecipient::Single(t) => {
                if let Some(connection) = self.connections.get_mut(&t) {
                    connection.outgoing_packets.push_back(packet_boxed);
                }
            }
            PacketRecipient::Exclude(t) => {
                let filtered = self.connections.iter_mut().filter(|(tok, _c)| tok.0 != t.0);
                for (_token, connection) in filtered {
                    connection.outgoing_packets.push_back(packet_boxed.clone());
                }
            }
            PacketRecipient::ExcludeMany(filter) => {
                let filtered = self
                    .connections
                    .iter_mut()
                    .filter(|(tok, _c)| !filter.contains(tok));
                for (_token, connection) in filtered {
                    connection.outgoing_packets.push_back(packet_boxed.clone());
                }
            }
            PacketRecipient::Include(targets) => {
                let filtered = self
                    .connections
                    .iter_mut()
                    .filter(|(tok, _c)| targets.contains(tok));
                for (_token, connection) in filtered {
                    connection.outgoing_packets.push_back(packet_boxed.clone());
                }
            }
        }
    }

    pub fn tick(&mut self) -> Vec<ServerEvent> {
        std::thread::sleep(std::time::Duration::from_millis(self.ms_sleep_time as u64));
        let timeout = std::time::Duration::from_millis(1);
        self.poll
            .poll(&mut self.events, Some(timeout))
            .unwrap_or_else(|e| panic!("Failed to poll for new events! {}", e));

        let mut net_events: Vec<ServerEvent> = Vec::new();
        for event in self.events.iter() {
            match event.token() {
                LOCAL_TOKEN => loop {
                    match self.tcp_listener.accept() {
                        Ok((mut socket, addr)) => {
                            self.token_counter += 1;
                            let token = Token(self.token_counter);

                            self.poll.registry().register(
                            &mut socket,
                            token,
                            Interest::READABLE.add(Interest::WRITABLE)
                        ).unwrap_or_else(|e| panic!("Failed to register poll for new connection (Token {}, Address {}). {}", token.0, addr, e));

                            self.connections
                                .insert(token, Connection::new(token, socket));

                            net_events.push(ServerEvent::ClientConnected(token, addr));
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            break;
                        }
                        Err(e) => println!("{}", e),
                    }
                },
                token => {
                    let conn: &mut Connection =
                        self.connections.get_mut(&token).unwrap_or_else(|| {
                            panic!(
                                "Attempted to handle socket event for non-existent connection {}!",
                                token.0
                            )
                        });

                    if event.is_readable() {
                        let buffer = &mut conn.buffer.data[conn.buffer.offset..];
                        loop {
                            match conn.socket.read(buffer) {
                                Ok(0) => {
                                    conn.disconnected = true;
                                    break;
                                }
                                Ok(read_bytes) => {
                                    conn.buffer.offset += read_bytes;
                                }
                                Err(e) => {
                                    if e.kind() == std::io::ErrorKind::WouldBlock {
                                        break;
                                    } else {
                                        eprintln!("Unexpected error when reading bytes from connection {}! {}", conn.token.0, e);
                                        conn.disconnected = true;
                                        break;
                                    }
                                }
                            }
                        }

                        while let Ok(header) = deserialize_packet_header(&mut conn.buffer) {
                            let packet_size = PACKET_HEADER_SIZE + (header.size as usize);
                            if conn.buffer.offset < packet_size {
                                break;
                            }

                            let bytes: &[u8] = &conn.buffer.data[PACKET_HEADER_SIZE..packet_size];
                            let body = bytes.to_vec();
                            conn.buffer.drain(packet_size);

                            let packet = Packet { header, body };

                            self.incoming_packets.push_back((token, packet));

                            net_events.push(ServerEvent::ReceivedPacket(conn.token, packet_size));
                        }
                    }

                    if event.is_writable() {
                        while let Some(packet) = conn.outgoing_packets.pop_front() {
                            let data = match serialize_packet(packet) {
                                Ok(d) => d,
                                Err(e) => {
                                    eprintln!("Failed to serialize packet! {}", e);
                                    continue;
                                }
                            };

                            match send_bytes(&mut conn.socket, &data) {
                                Ok(sent_bytes) => {
                                    net_events.push(ServerEvent::SentPacket(token, sent_bytes));
                                }
                                Err(e) => {
                                    eprintln!(
                                        "Unexpected error when sending bytes to connection {}! {}",
                                        conn.token.0, e
                                    );
                                    conn.disconnected = true;
                                    break;
                                }
                            }
                        }
                    }

                    self.poll
                        .registry()
                        .reregister(
                            &mut conn.socket,
                            conn.token,
                            Interest::READABLE.add(Interest::WRITABLE),
                        )
                        .unwrap_or_else(|e| {
                            panic!(
                                "Failed to reregister poll for connection (Token {}). {}",
                                token.0, e
                            )
                        });
                }
            }
        }

        for (tok, _) in self.connections.iter().filter(|&(_, c)| c.disconnected) {
            net_events.push(ServerEvent::ClientDisconnected(*tok));
        }

        self.connections.retain(|_, v| !v.disconnected);

        net_events
    }
}
