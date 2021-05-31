use stratepig_core::Packet;

use crate::constants;
use crate::gameroom;
use crate::gameroom::{GameMode, GameRoomError};
use crate::player::{PlayerRole, RoomPlayer};
use crate::GameServer;
mod send;
mod settings;

impl GameServer {
    pub async fn handle_game_request(&mut self, id: usize, mut packet: Packet) {
        // Macros are better for this than async closures
        macro_rules! reject {
            () => {
                self.fail_create_game(id).await;
                return;
            };
        }

        let id_check = packet.read_string().unwrap_or(String::new());
        let hosting = packet.read_bool().unwrap_or(false);
        let username = packet.read_string().unwrap_or(String::new());
        let icon = packet.read_i32().unwrap_or(0);
        let code = packet.read_string();
        let data_null = packet.read_bool().unwrap_or(false); // If packet specifies preferences in settings

        if id.to_string() != id_check {
            self.fail_create_game(id).await;
            return;
        }

        if username == "" || username.len() > constants::MAX_USERNAME_LENGTH as usize {
            reject!();
        }
        if icon < 0 || icon >= 13 {
            reject!();
        }

        if hosting {
            let room = self.create_room_from_data(data_null, &mut packet).await;
            if let Err(err) = room {
                let err = String::from(err);
                drop(room); // We need to drop the room first before we immutably borrow self

                if let "" = &err[..] {
                    reject!();
                } else {
                    self.err_join_game(id, &err).await;
                    return;
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
                username,
                icon as u8,
                client,
            ));

            let reference = self.get_room(room_id).unwrap();

            self.initialize_player(id, PlayerRole::One).await;
            self.room_player_add(&reference).await;
            self.send_game_info(&reference, id).await;
        } else {
            let code = code.unwrap_or(String::new());
            let room_join = self.try_join_room(&code);
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
                    return;
                }
                Ok(_) => {
                    let found = room_join.unwrap();
                    let read = found.inner();
                    let room_id = read.id;
                    let safe_username = self.generate_safe_username(&found, &username);

                    drop(read);
                    drop(found);
                    let client = self.all_clients.get_mut(id).unwrap();

                    client.set_game_room(room_id);
                    client.room_player = Some(RoomPlayer::new(
                        PlayerRole::Two,
                        safe_username,
                        icon as u8,
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
                    self.send_game_info(&reference, id).await;
                }
            }
        }
    }

    pub async fn handle_ready_state_change(&mut self, id: usize, mut packet: Packet) {
        let id_check = packet.read_string().unwrap_or(String::new());
        let ready = packet.read_bool().unwrap_or(false);

        if id.to_string() != id_check {
            return;
        }
        let ctx = self.get_context(id);
        if let None = ctx {
            return;
        }
        let (_client, room) = ctx.unwrap();
        let room_id = room.id();

        if room.inner().in_game {
            return;
        }

        drop(room);

        self.all_clients
            .get_mut(id)
            .unwrap()
            .room_player
            .as_mut()
            .unwrap()
            .ready = ready;
        let reference = self.get_room(room_id).unwrap();
        self.room_update_ready_state(&reference, id, ready).await;

        if ready {
            if self.config.one_player {
                reference.start(self).await;
            } else {
                // If there is any better way to do this, please let me know
                if let Some(res) = self.get_other_player(&reference, id) {
                    if let Some(player) = &res.room_player {
                        if player.ready {
                            reference.start(self).await;
                        }
                    }
                }
            }
        } else {
            reference.cancel_start();

            let mut packet = Packet::new_id(crate::ServerMessage::RoomTimerUpdate as i32);
            packet.write_i64(-1);
            self.message_room(&reference, packet).await;
        }
    }

    pub async fn handle_update_icon(&mut self, id: usize, mut packet: Packet) {
        let id_check = packet.read_string().unwrap_or(String::new());
        let icon = packet.read_u32().unwrap_or(0);

        if id.to_string() != id_check {
            return;
        }

        let ctx = self.get_context(id);
        if let None = ctx {
            return;
        }
        let (_client, room) = ctx.unwrap();
        let room_id = room.id();
        drop(room);

        self.all_clients
            .get_mut(id)
            .unwrap()
            .room_player
            .as_mut()
            .unwrap()
            .icon = icon as u8;
        let reference = self.get_room(room_id).unwrap();

        if icon > 12 {
            return;
        }

        self.update_icon(&reference, id, icon).await;
    }

    pub async fn handle_settings_value_update(&mut self, id: usize, mut packet: Packet) {
        let id_check = packet.read_string().unwrap_or(String::new());
        let settings_id = packet.read_u32().unwrap_or(0);
        let increased = packet.read_bool().unwrap_or(true);

        if id.to_string() != id_check {
            return;
        }

        let ctx = self.get_context(id);
        if let None = ctx {
            return;
        }
        let (client, room) = ctx.unwrap();
        let room_id = room.id();

        if client.player.as_ref().unwrap().role == PlayerRole::One {
            let key = &(settings_id as u8);

            if settings_id <= 0 {
                let mut current_value = room.inner().settings.game_mode as u8;
                if increased {
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
                self.update_settings_value(&reference, settings_id, current_value as u32)
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
            } else if settings_id <= 3 {
                let mut current_value = match settings_id {
                    1 => room.inner().settings.placement_time,
                    2 => room.inner().settings.turn_time,
                    3 => room.inner().settings.buffer_time,
                    _ => 0,
                } as i32;

                let group = gameroom::SETTINGS_GROUPS.get(key).unwrap();

                if increased {
                    current_value += group.interval as i32;
                    if current_value as i32 > group.max_val {
                        if group.loopable {
                            current_value = group.min_val;
                        } else {
                            return;
                        }
                    }
                } else {
                    current_value -= group.interval as i32;
                    if (current_value as i32) < group.min_val {
                        if group.loopable {
                            current_value = group.max_val;
                        } else {
                            return;
                        }
                    }
                }

                match settings_id {
                    1 => room.get().write().unwrap().settings.placement_time = current_value as u32,
                    2 => room.get().write().unwrap().settings.turn_time = current_value as u32,
                    3 => room.get().write().unwrap().settings.buffer_time = current_value as u32,
                    _ => {}
                };

                drop(room);
                let reference = self.get_room(room_id).unwrap();

                self.update_settings_value(&reference, settings_id, current_value as u32)
                    .await;
            }
        }
    }
}
