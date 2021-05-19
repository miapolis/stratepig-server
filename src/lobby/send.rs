use crate::*;
use std::collections::HashMap;

impl GameServer {
    pub async fn fail_create_game(&self, id: usize) {
        let packet = Packet::new_id(ServerMessage::FailCreateGame as i32);
        self.message_one(id, packet).await;
    }

    pub async fn initialize_player(&self, id: usize, role: u8) {
        let mut packet = Packet::new_id(ServerMessage::ClientInfo as i32);
        packet.write_u32(role as u32);
        self.message_one(id, packet).await;
    }

    pub async fn room_player_add(&self, room: &GameRoom) {
        let read = room.inner();

        for id in read.client_ids.iter() {
            let client = self.all_clients.get(*id).unwrap();
            let mut packet = Packet::new_id(ServerMessage::RoomPlayerAdd as i32);
            packet.write_str(&client.id.to_string());
            packet.write_i32(read.client_ids.len() as i32);

            let room_player = client.room_player.as_ref().unwrap();
            packet.write_str(&room_player.username);
            packet.write_bool(room_player.ready);
            packet.write_i32(room_player.icon as i32);

            self.message_room(room, packet).await;
        }
    }

    pub async fn client_disconnected(&self, id: usize, room: &GameRoom) {
        let mut packet = Packet::new_id(ServerMessage::ClientDisconnect as i32);
        packet.write_str(&id.to_string());
        packet.write_u64(0); // TODO: actually calculate elapsed unix timestamp
        self.message_room(room, packet).await;
    }

    pub async fn send_game_info(&self, id: usize, room: &GameRoom) {
        let room = room.inner();

        let mut packet = Packet::new_id(ServerMessage::GameInfo as i32);
        packet.write_str(&room.code);
        packet.write_i32(room.settings.game_mode as i32);
        packet.write_u32(room.settings.placement_time);
        packet.write_u32(room.settings.turn_time);
        packet.write_u32(room.settings.buffer_time);

        packet.write_u32(room.settings.pig_config.len() as u32);

        for item in &room.settings.pig_config {
            packet.write_u32(*item.0 as u32);
            packet.write_u32(*item.1 as u32);
        }

        self.message_one(id, packet).await;
    }

    pub async fn err_join_game(&self, id: usize, message: &str) {
        let mut packet = Packet::new_id(ServerMessage::ErrorJoinGame as i32);
        packet.write_str(message);
        self.message_one(id, packet).await;
    }

    pub async fn room_update_ready_state(&self, room: &GameRoom, id: usize, ready: bool) {
        let mut packet = Packet::new_id(ServerMessage::RoomPlayerUpdatedReadyState as i32);
        packet.write_str(&id.to_string());
        packet.write_bool(ready);

        self.message_room(room, packet).await;
    }

    pub async fn update_room_timer(&self, room: &GameRoom, seconds: i32) {
        let mut packet = Packet::new_id(ServerMessage::RoomTimerUpdate as i32);
        packet.write_i32(seconds);
        self.message_room(room, packet).await;
    }

    pub async fn update_icon(&self, room: &GameRoom, id: usize, icon: u32) {
        let mut packet = Packet::new_id(ServerMessage::UpdatedPigIcon as i32);
        packet.write_str(&id.to_string());
        packet.write_u32(icon);
        self.message_room(room, packet).await;
    }

    pub async fn update_settings_value(&self, room: &GameRoom, id: u32, value: u32) {
        let mut packet = Packet::new_id(ServerMessage::SettingsValueChanged as i32);
        packet.write_u32(id);
        packet.write_u32(value);
        self.message_room(room, packet).await;
    }

    pub async fn update_config_bulk(&self, room: &GameRoom, config: HashMap<u8, u8>) {
        let read = room.inner();

        let mut packet = Packet::new_id(ServerMessage::PigConfigValueChanged as i32);
        packet.write_u32(read.settings.turn_time);
        packet.write_u32(read.settings.buffer_time);

        packet.write_u32(config.keys().len() as u32);

        for (key, value) in config {
            packet.write_u32(key as u32);
            packet.write_u32(value as u32);
        }

        self.message_room(room, packet).await;
    }
}
