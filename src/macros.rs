#[macro_export]
macro_rules! message_room {
    ($server:expr, $room:expr, $packet:expr) => {
        let lock = $server.lock();

        $packet.write_length();
        let bytes = $packet.to_array();

        for id in $room.read().unwrap().client_ids.iter() {
            lock.message_one(*id, bytes);
        }
        drop(lock);
    };
}
