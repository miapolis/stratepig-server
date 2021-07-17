use std::convert::TryFrom;
use stratepig_core::{Packet, PacketBody};

use crate::constants;
use crate::gameroom;
use crate::gameroom::{GameMode, GameRoomError};
use crate::packet::{
    GameRequestDefaultPacket, GameRequestFullPacket, LeaveGamePacket, RoomTimerUpdatePacket,
    UpdatePigIconPacket, UpdateReadyStatePacket, UpdateSettingsValue,
};
use crate::player::{PlayerRole, RoomPlayer};
use crate::GameServer;
use crate::StratepigError;
mod send;
mod settings;

impl GameServer {
    pub async fn handle_game_request(
        &mut self,
        id: usize,
        mut packet: Packet,
    ) -> Result<(), StratepigError> {
        let data = GameRequestDefaultPacket::deserialize(&packet.body)?;
        // Macros are better for this than async closures
        macro_rules! reject {
            () => {
                self.fail_create_game(id).await;
                return Err(StratepigError::with("failed to create game"));
            };
        }

        if data.my_id != id.to_string() {
            return Err(StratepigError::AssumeWrongId);
        }
        if data.username.trim() == "".to_owned()
            || data.username.len() > constants::MAX_USERNAME_LENGTH as usize
        {
            reject!();
        }
        if data.icon < 0 || data.icon >= 13 {
            reject!();
        }

        if data.is_hosting {
            let room = self
                .create_room_from_data(data.data_null, &mut packet)
                .await;

            if let Err(err) = room {
                let err = String::from(err);
                drop(room); // We need to drop the room first before we immutably borrow self
                if let "" = &err[..] {
                    reject!();
                } else {
                    self.err_join_game(id, &err).await;
                    return Err(StratepigError::with("failed to create game"));
                }
            }

            let room = room.unwrap();
            let room_id = room.id();
            let mut write = room.get().write().unwrap();
            write.client_ids.push(id);

            drop(write);
            drop(room);

            let client = self.all_clients.get_mut(id).unwrap();
            client.set_game_room(room_id);
            client.room_player = Some(RoomPlayer::new(
                PlayerRole::One,
                data.username,
                data.icon as u8,
                client,
            ));

            let reference = self.get_room(room_id).unwrap();

            self.initialize_player(id, PlayerRole::One).await;
            self.room_player_add(&reference).await;
            self.send_game_info(&reference, Some(id)).await;
        } else {
            let room_join = self.try_join_room(&data.code);
            match room_join {
                Err(err) => {
                    match err {
                        GameRoomError::NotFound => {
                            self.err_join_game(id, "Could not find the game you were looking for.")
                                .await
                        }
                        GameRoomError::Started => {
                            self.err_join_game(id, "That game has already started.")
                                .await
                        }
                        GameRoomError::Full => self.err_join_game(id, "That game is full.").await,
                    }
                    return Ok(());
                }
                Ok(_) => {
                    let found = room_join.unwrap();
                    let read = found.inner();
                    let room_id = read.id;
                    let safe_username = self.generate_safe_username(&found, &data.username);

                    drop(read);
                    drop(found);

                    let client = self.all_clients.get_mut(id).unwrap();
                    client.set_game_room(room_id);
                    client.room_player = Some(RoomPlayer::new(
                        PlayerRole::Two,
                        safe_username,
                        data.icon as u8,
                        client,
                    ));

                    self.get_room(room_id)
                        .unwrap()
                        .get()
                        .write()
                        .unwrap()
                        .client_ids
                        .push(id);

                    let reference = self.get_room(room_id).unwrap();

                    self.initialize_player(id, PlayerRole::Two).await;
                    self.room_player_add(&reference).await;
                    self.send_game_info(&reference, Some(id)).await;
                }
            }
        }

        Ok(())
    }

    pub async fn handle_client_leave(
        &mut self,
        id: usize,
        mut packet: Packet,
    ) -> Result<(), StratepigError> {
        let data = LeaveGamePacket::deserialize(&packet.body)?;

        if id.to_string() != data.my_id {
            return Err(StratepigError::AssumeWrongId);
        }

        let ctx = self.get_context(id);
        if let None = ctx {
            return Err(StratepigError::MissingContext);
        }
        let (_client, room) = ctx.unwrap();
        let room_id = room.id();
        drop(room);

        self.handle_client_disconnect(room_id, id).await;
        Ok(())
    }

    pub async fn handle_ready_state_change(
        &mut self,
        id: usize,
        mut packet: Packet,
    ) -> Result<(), StratepigError> {
        let data = UpdateReadyStatePacket::deserialize(&packet.body)?;

        if id.to_string() != data.my_id {
            return Err(StratepigError::AssumeWrongId);
        }

        let ctx = self.get_context(id);
        if let None = ctx {
            return Err(StratepigError::MissingContext);
        }
        let (_client, room) = ctx.unwrap();
        let room_id = room.id();

        if room.inner().in_game {
            return Err(StratepigError::with("cannot update ready state in game"));
        }

        drop(room);

        self.all_clients
            .get_mut(id)
            .unwrap()
            .room_player
            .as_mut()
            .unwrap()
            .ready = data.ready;
        let reference = self.get_room(room_id).unwrap();
        self.room_update_ready_state(&reference, id, data.ready)
            .await;

        if data.ready {
            if self.config.one_player {
                reference.start(self, 1).await;
            } else {
                // If there is any better way to do this, please let me know
                if let Some(res) = self.get_other_player(&reference, id) {
                    if let Some(player) = &res.room_player {
                        if player.ready {
                            reference.start(self, 5).await;
                        }
                    }
                }
            }
        } else {
            reference.cancel_start();

            let packet = RoomTimerUpdatePacket { timestamp: -1 };
            self.message_room(&reference, packet).await;
        }

        Ok(())
    }

    pub async fn handle_update_icon(
        &mut self,
        id: usize,
        mut packet: Packet,
    ) -> Result<(), StratepigError> {
        let data = UpdatePigIconPacket::deserialize(&packet.body)?;

        if id.to_string() != data.my_id {
            return Err(StratepigError::AssumeWrongId);
        }

        let ctx = self.get_context(id);
        if let None = ctx {
            return Err(StratepigError::MissingContext);
        }
        let (_client, room) = ctx.unwrap();
        let room_id = room.id();
        drop(room);

        if data.icon > 12 {
            return Err(StratepigError::with("icon out-of-bounds"));
        }

        self.all_clients
            .get_mut(id)
            .unwrap()
            .room_player
            .as_mut()
            .unwrap()
            .icon = data.icon as u8;
        let reference = self.get_room(room_id).unwrap();

        self.update_icon(&reference, id, data.icon).await;

        Ok(())
    }

    pub async fn handle_settings_value_update(
        &mut self,
        id: usize,
        mut packet: Packet,
    ) -> Result<(), StratepigError> {
        let data = UpdateSettingsValue::deserialize(&packet.body)?;

        if id.to_string() != data.my_id {
            return Err(StratepigError::AssumeWrongId);
        }

        let ctx = self.get_context(id);
        if let None = ctx {
            return Err(StratepigError::MissingContext);
        }
        let (client, room) = ctx.unwrap();
        let room_id = room.id();

        if client.player.as_ref().unwrap().role == PlayerRole::One {
            let key = &(u8::try_from(data.settings_id).unwrap_or(0));

            if data.settings_id <= 0 {
                let mut current_value = room.inner().settings.game_mode as u8;
                if data.increased {
                    current_value += 1;
                    if current_value > GameMode::MAX {
                        current_value = 1;
                    }
                } else {
                    current_value -= 1;
                    if current_value < 1 {
                        current_value = GameMode::MAX;
                    }
                }

                let current_type = GameMode::from(current_value);
                room.get().write().unwrap().settings.game_mode = current_type;
                drop(room);

                let reference = self.get_room(room_id).unwrap();
                self.update_settings_value(&reference, data.settings_id, current_value as u32)
                    .await;

                if current_type != GameMode::Custom {
                    let config = gameroom::get_pig_config_for_mode(current_type).unwrap();
                    let settings_vars = gameroom::get_settings_vars(current_type);

                    let mut write = reference.get().write().unwrap();
                    write.settings.turn_time = settings_vars.turn_time;
                    write.settings.buffer_time = settings_vars.buffer_time;
                    write.settings.pig_config = config.clone();
                    drop(write);

                    self.update_config_bulk(&reference, config).await;
                }
            } else if data.settings_id <= 3 {
                let mut current_value = match data.settings_id {
                    1 => room.inner().settings.placement_time,
                    2 => room.inner().settings.turn_time,
                    3 => room.inner().settings.buffer_time,
                    _ => 0,
                } as i32;

                let group = gameroom::SETTINGS_GROUPS.get(key).unwrap();

                if data.increased {
                    current_value += group.interval as i32;
                    if current_value as i32 > group.max_val {
                        if group.loopable {
                            current_value = group.min_val;
                        } else {
                            return Ok(());
                        }
                    }
                } else {
                    current_value -= group.interval as i32;
                    if (current_value as i32) < group.min_val {
                        if group.loopable {
                            current_value = group.max_val;
                        } else {
                            return Ok(());
                        }
                    }
                }

                match data.settings_id {
                    1 => room.get().write().unwrap().settings.placement_time = current_value as u32,
                    2 => room.get().write().unwrap().settings.turn_time = current_value as u32,
                    3 => room.get().write().unwrap().settings.buffer_time = current_value as u32,
                    _ => {}
                };

                drop(room);
                let reference = self.get_room(room_id).unwrap();

                self.update_settings_value(&reference, data.settings_id, current_value as u32)
                    .await;
            }
        }

        Ok(())
    }
}
