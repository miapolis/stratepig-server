#[macro_export]
macro_rules! message_room {
    ($handler:expr, $room:expr, $packet:expr) => {{
        use crate::Endpoint;
        use stratepig_core::serialize_packet;

        let endpoints: Vec<Endpoint> = $room
            .read()
            .unwrap()
            .client_ids
            .clone()
            .into_iter()
            .map(|x| x.1)
            .collect();

        let bytes = &serialize_packet(Box::new($packet)).unwrap();
        let handler = $handler.lock();
        for endpoint in endpoints.into_iter() {
            handler.network().send(endpoint, bytes);
        }
    }};
}
