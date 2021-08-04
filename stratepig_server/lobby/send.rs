use crate::*;
use std::collections::HashMap;

impl GameServer {
    pub async fn fail_create_game(&self, id: usize) {
        let packet = FailCreateGamePacket {};
        self.message_one(id, packet).await;
    }

    pub async fn initialize_player(&self, id: usize, role: PlayerRole) {
        let packet = ClientInfoPacket { role: role as u32 };
        self.message_one(id, packet).await;
    }

    pub async fn room_player_add(&self, room: &GameRoom) {
        let read = room.inner();

        for id in read.client_ids.iter() {
            let client = self.all_clients.get(&id.0).unwrap();
            let room_player = client.room_player.as_ref().unwrap();

            let packet = RoomPlayerAddPacket {
                id: client.id.to_string(),
                client_count: read.client_ids.len() as i32,
                username: room_player.username.clone(),
                ready: room_player.ready,
                icon: room_player.icon as i32,
            };

            self.message_room(room, packet).await;
        }
    }

    pub async fn client_disconnected(&self, room: &GameRoom, id: usize) {
        let packet = ClientDisconnectPacket {
            id: id.to_string(),
            timestamp: 0, // TODO: actually calculate elapsed unix timestamp
        };
        self.message_room(room, packet).await;
    }

    pub async fn send_game_info(&self, room: &GameRoom, id: Option<usize>) {
        let inner = room.inner();

        let packet = GameInfoPacket {
            code: inner.code.clone(),
            game_mode: inner.settings.game_mode as i32,
            placement_time: inner.settings.placement_time,
            turn_time: inner.settings.turn_time,
            buffer_time: inner.settings.buffer_time,
            pig_config: inner
                .settings
                .pig_config
                .iter()
                .map(|(key, value)| {
                    return (*key as u32, *value as u32);
                })
                .collect(),
        };

        if let Some(id) = id {
            self.message_one(id, packet).await;
        } else {
            self.message_room(room, packet).await;
        }
    }

    pub async fn err_join_game(&self, id: usize, message: &str) {
        let packet = ErrJoinGamePacket {
            msg: message.to_owned(),
        };
        self.message_one(id, packet).await;
    }

    pub async fn room_update_ready_state(&self, room: &GameRoom, id: usize, ready: bool) {
        let packet = RoomPlayerUpdatedReadyStatePacket {
            id: id.to_string(),
            ready,
        };

        self.message_room(room, packet).await;
    }

    pub async fn update_room_timer(&self, room: &GameRoom, seconds: i32) {
        let packet = RoomTimerUpdatePacket {
            timestamp: seconds as i64,
        };
        self.message_room(room, packet).await;
    }

    pub async fn update_icon(&self, room: &GameRoom, id: usize, icon: u32) {
        let packet = UpdatedPigIconPacket {
            id: id.to_string(),
            icon: icon as i32,
        };

        self.message_room(room, packet).await;
    }

    pub async fn update_settings_value(&self, room: &GameRoom, id: u32, value: u32) {
        let packet = SettingsValueChangedPacket { id, value };
        self.message_room(room, packet).await;
    }

    pub async fn update_pig_item(&self, room: &GameRoom, pig: u32, amount: u32) {
        let packet = PigItemValueChangedPacket { pig, amount };
        self.message_room(room, packet).await;
    }

    pub async fn update_config_bulk(
        &self,
        room: &GameRoom,
        config: HashMap<stratepig_game::Pig, u8>,
    ) {
        let read = room.inner();

        let packet = PigConfigValueChangedPacket {
            turn_time: read.settings.turn_time,
            buffer_time: read.settings.buffer_time,
            pig_config: config
                .iter()
                .map(|(key, value)| {
                    return (*key as u32, *value as u32);
                })
                .collect(),
        };

        self.message_room(room, packet).await;
    }
}
