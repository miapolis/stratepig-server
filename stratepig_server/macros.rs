#[macro_export]
macro_rules! message_room {
    ($server:expr, $room:expr, $packet:expr) => {{
        use crate::PacketRecipient;
        use crate::Token;

        let tokens: Vec<Token> = $room
            .read()
            .unwrap()
            .client_ids
            .clone()
            .into_iter()
            .map(|x| Token(x))
            .collect();
        $server
            .lock()
            .send(PacketRecipient::Include(tokens), $packet);
    }};
}
