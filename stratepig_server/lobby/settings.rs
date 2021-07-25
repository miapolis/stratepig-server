use std::collections::HashMap;
use std::convert::TryFrom;

use stratepig_core::{Packet, PacketBody};
use stratepig_game::Pig;

use crate::gameroom::{self, GameMode, GameRoom, SettingsGroup};
use crate::packet::GameRequestFullPacket;
use crate::GameServer;

impl GameServer {
    pub async fn create_room_from_data(
        &mut self,
        data_null: bool,
        packet: &mut Packet,
    ) -> Result<impl std::ops::Deref<Target = GameRoom> + '_, &str> {
        macro_rules! err {
            () => {
                return Err("invalid config");
            };
        }

        if !data_null {
            let data = GameRequestFullPacket::deserialize(&packet.body).or_else(|_e| {
                return Err("invalid packet");
            })?;

            let game_mode = match data.game_mode {
                1 => GameMode::Original,
                2 => GameMode::Infiltrator,
                3 => GameMode::Duel,
                4 => GameMode::Custom,
                _ => GameMode::Original,
            };

            let placement_group = gameroom::SETTINGS_GROUPS.get(&1).unwrap();
            let placement_secs = sanitize_setting(data.placement_secs as u32, placement_group);
            let turn_group = gameroom::SETTINGS_GROUPS.get(&2).unwrap();
            let turn_secs = sanitize_setting(data.turn_secs as u32, turn_group);
            let buffer_group = gameroom::SETTINGS_GROUPS.get(&3).unwrap();
            let buffer_secs = sanitize_setting(data.buffer_secs as u32, buffer_group);

            let mut pig_config: HashMap<Pig, u8> = data
                .pig_config
                .into_iter()
                .map(|(pig, amt)| {
                    let pig = Pig::from(pig as u32);
                    let amt = u8::try_from(amt).unwrap_or(0);
                    return (pig, amt);
                })
                .collect();

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
            write.settings.placement_time = placement_secs;
            write.settings.turn_time = turn_secs;
            write.settings.buffer_time = buffer_secs;
            write.settings.pig_config = pig_config;
            drop(write);

            return Ok(room);
        } else {
            let room = self.new_room()?;
            room.load_default_settings();

            return Ok(room);
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
