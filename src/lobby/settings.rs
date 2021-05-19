use crate::board::Pig;
use crate::gameroom::{self, GameMode, GameRoom, SettingsGroup};
use crate::GameServer;
use crate::Packet;
use std::collections::HashMap;

impl GameServer {
    pub async fn create_room_from_data(
        &mut self,
        data_null: bool,
        packet: &mut Packet,
    ) -> Result<impl std::ops::Deref<Target = GameRoom> + '_, &str> {
        macro_rules! err {
            () => {
                return Err("Invalid config.");
            };
        }

        if !data_null {
            let int_type = packet.read_u32().unwrap_or(1);
            let mut placement_seconds = packet.read_u32().unwrap_or(300);
            let mut turn_seconds = packet.read_u32().unwrap_or(15);
            let mut buffer_seconds = packet.read_u32().unwrap_or(300);
            let amount_of_pigs = packet.read_u32().unwrap_or(40);

            let game_mode = match int_type {
                1 => GameMode::Original,
                2 => GameMode::Infiltrator,
                3 => GameMode::Duel,
                4 => GameMode::Custom,
                _ => GameMode::Original,
            };

            let placement_group = gameroom::SETTINGS_GROUPS.get(&1).unwrap();
            placement_seconds = sanitize_setting(placement_seconds, placement_group);
            let turn_group = gameroom::SETTINGS_GROUPS.get(&2).unwrap();
            turn_seconds = sanitize_setting(turn_seconds, turn_group);
            let buffer_group = gameroom::SETTINGS_GROUPS.get(&3).unwrap();
            buffer_seconds = sanitize_setting(buffer_seconds, buffer_group);

            let mut pig_config = HashMap::new();

            for _ in 0..amount_of_pigs {
                let pig = packet.read_u32();
                let val;
                match pig {
                    Ok(v) => val = v,
                    Err(_) => break,
                }
                let amt = packet.read_u32().unwrap_or(0) as u8;

                pig_config.insert(Pig::from(val), amt);
            }

            for i in 0..13 {
                let key = Pig::from(i);
                if !pig_config.contains_key(&key) {
                    pig_config.insert(key, 0); // Fill empty spaces with 0
                }
            }

            pig_config = match gameroom::get_pig_config_for_mode(game_mode) {
                Some(data) => data,
                None => pig_config,
            };

            if game_mode == GameMode::Custom {
                if pig_config.keys().len() > 13 {
                    err!();
                }
                for key in pig_config.keys() {
                    if *key == Pig::Empty {
                        err!();
                    }
                }

                let mut total = 0;
                for val in pig_config.values() {
                    total += *val;
                }
                if total > 40 || total <= 0 {
                    err!();
                }
            }

            let room = self.new_room()?;
            let mut write = room.get().write().unwrap();

            write.settings.game_mode = game_mode;
            write.settings.placement_time = placement_seconds;
            write.settings.turn_time = turn_seconds;
            write.settings.buffer_time = buffer_seconds;
            write.settings.pig_config = pig_config;
            drop(write);

            Ok(room)
        } else {
            let room = self.new_room()?;
            room.load_default_settings();
            Ok(room)
        }
    }
}

fn sanitize_setting(mut provided: u32, setting: &SettingsGroup) -> u32 {
    if provided > setting.max_val as u32
        || (provided < setting.min_val as u32)
        || (provided % setting.interval != 0)
    {
        provided = setting.default as u32;
    }
    provided
}
