use std::collections::VecDeque;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::ops::Fn;
use std::sync::atomic::{self, Ordering};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread::{self};

use vec_map::VecMap;

type ClientConn = (Receiver<io::Result<Vec<u8>>>, TcpStream);

pub struct Server {
    pub connections: Arc<Mutex<VecMap<ClientConn>>>,
    pub client_count: Arc<atomic::AtomicUsize>,
    pub new_r: Receiver<usize>,
}

impl Server {
    /// Creates a new server
    pub fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:32500").unwrap();
        let connections = Arc::new(Mutex::new(VecMap::new()));
        let client_count = Arc::new(atomic::AtomicUsize::new(0));
        let (new_s, new_r) = channel();
        {
            let listener = listener.try_clone().unwrap();
            let connections = connections.clone();
            let client_count = client_count.clone();

            thread::spawn(move || {
                let mut id = 0;
                let free_ids = Arc::new(Mutex::new(VecDeque::new()));
                for conn in listener.incoming() {
                    if let Ok(conn) = conn {
                        let free_ids = free_ids.clone();
                        let new_id = match free_ids.lock().unwrap().pop_front() {
                            Some(id) => id,
                            None => {
                                id += 1;
                                id
                            }
                        };

                        let connections = connections.clone();
                        let connections_clone = connections.clone();
                        let client_count = client_count.clone();
                        let conn_reader = conn.try_clone().unwrap();
                        let (ds, dr) = channel();
                        let new_s = new_s.clone();

                        thread::spawn(move || {
                            let mut reader = BufReader::new(conn_reader);
                            let my_id = new_id;
                            // Increment the client count by one
                            client_count
                                .store(client_count.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
                            // Send to the channel that a client has connected
                            new_s.send(my_id).unwrap();

                            loop {
                                let result = match reader.fill_buf() {
                                    Ok(data) if data.len() == 0 => Some(0),
                                    Ok(data) => {
                                        ds.send(Ok(data.to_vec())).unwrap();
                                        Some(data.len())
                                    }
                                    Err(e) => {
                                        ds.send(Err(e)).unwrap();
                                        None
                                    }
                                };

                                if let Some(read) = result {
                                    if read > 0 {
                                        reader.consume(read);
                                    } else {
                                        drop(ds);
                                        free_ids.lock().unwrap().push_back(my_id);
                                        break;
                                    }
                                }
                            }

                            client_count
                                .store(client_count.load(Ordering::Relaxed) - 1, Ordering::Relaxed);
                            connections.lock().unwrap().remove(my_id);
                        });

                        connections_clone.lock().unwrap().insert(new_id, (dr, conn));
                    }
                }
            });
        }

        Self {
            connections,
            client_count,
            new_r,
        }
    }

    /// Send a message to all connected clients
    pub fn message_all(&self, data: &[u8]) {
        self.message(|_| true, data)
    }

    /// Send a message to a single client
    pub fn message_one(&self, client: usize, data: &[u8]) {
        self.message(|id| id == client, data)
    }

    /// Send a message to every client but one
    pub fn message_except(&self, client: usize, data: &[u8]) {
        self.message(|id| id != client, data)
    }

    // Custom message based on a predicate
    pub fn message<P>(&self, predicate: P, data: &[u8])
    where
        P: Fn(usize) -> bool,
    {
        for (_, &mut (_, ref mut conn)) in self
            .connections
            .lock()
            .unwrap()
            .iter_mut()
            .filter(|&(id, _)| predicate(id))
        {
            conn.write_all(data).unwrap();
        }
    }

    pub fn scan(&mut self) -> (usize, ServerEvent) {
        loop {
            match self.new_r.try_recv() {
                Ok(id) => return (id, ServerEvent::Connected),
                Err(e) if e == TryRecvError::Empty => break,
                Err(e) if e == TryRecvError::Disconnected => {
                    panic!("Tried to check for new clients on disconnected channel!");
                }
                Err(_) => unimplemented!(),
            }
        }

        let mut results = Vec::with_capacity(self.connections.lock().unwrap().len());

        for (id, &mut (ref mut dr, _)) in self.connections.lock().unwrap().iter_mut() {
            match dr.try_recv() {
                Ok(Ok(data)) => results.push((id, ServerEvent::Data(data))),
                Ok(Err(err)) => results.push((id, ServerEvent::IoError(err))),
                Err(TryRecvError::Empty) => {} // Do nothing
                Err(TryRecvError::Disconnected) => results.push((id, ServerEvent::Disconnected)),
            }
        }

        for (id, result) in results.into_iter() {
            if let ServerEvent::Disconnected = result {
                self.connections.lock().unwrap().remove(id);
            }
            return (id, result);
        }

        (0, ServerEvent::Empty)
    }

    pub fn disconnect_client(&mut self, id: usize) {
        self.client_count.store(
            self.client_count.load(Ordering::Relaxed) - 1,
            Ordering::Relaxed,
        );
        self.connections.lock().unwrap().remove(id);
    }
}

pub enum ServerEvent {
    /// The client sent data
    Data(Vec<u8>),
    /// An IO error occurred while scanning
    IoError(io::Error),
    /// A new client has connected
    Connected,
    /// A client has disconnected
    Disconnected,
    /// Nothing occured
    Empty,
}
