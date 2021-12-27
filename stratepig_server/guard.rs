use crate::error::StratepigError;
use crate::packet::BaseGuardPacket;
use crate::GameServer;
use dyn_clone::{clone_trait_object, DynClone};
use stratepig_core::{Packet, PacketBody};

pub trait Guard: DynClone + 'static {
    fn guard(&self, id: usize, packet: Packet, server: &GameServer) -> Result<(), StratepigError>;
    fn name(&self) -> &'static str;
}

clone_trait_object!(Guard);

#[derive(Clone, Debug)]
pub struct InRoomGuard;

impl Guard for InRoomGuard {
    fn guard(&self, id: usize, packet: Packet, server: &GameServer) -> Result<(), StratepigError> {
        let data = BaseGuardPacket::deserialize(&packet.body)?;
        if id.to_string() != data.my_id {
            return Err(StratepigError::AssumeWrongId);
        }

        let ctx = server.get_context(id);
        if let None = ctx {
            return Err(StratepigError::MissingContext);
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "Must be in a room"
    }
}

#[derive(Clone, Debug)]
pub struct InGameGuard;

impl Guard for InGameGuard {
    fn guard(&self, id: usize, packet: Packet, server: &GameServer) -> Result<(), StratepigError> {
        let data = BaseGuardPacket::deserialize(&packet.body)?;
        if id.to_string() != data.my_id {
            return Err(StratepigError::AssumeWrongId);
        }

        let ctx = server.get_context(id);
        if let None = ctx {
            return Err(StratepigError::MissingContext);
        }

        let (client, _room) = ctx.unwrap();

        if client.player.as_ref().is_none() {
            return Err(StratepigError::with("missing player object on client"));
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "Must be in a game"
    }
}

#[derive(Clone, Debug)]
pub struct InGameStrictGuard;

impl Guard for InGameStrictGuard {
    fn guard(&self, id: usize, packet: Packet, server: &GameServer) -> Result<(), StratepigError> {
        let data = BaseGuardPacket::deserialize(&packet.body)?;
        if id.to_string() != data.my_id {
            return Err(StratepigError::AssumeWrongId);
        }

        let ctx = server.get_context(id);
        if let None = ctx {
            return Err(StratepigError::MissingContext);
        }

        let (client, room) = ctx.unwrap();

        if client.player.as_ref().is_none() {
            return Err(StratepigError::with("missing player object on client"));
        }

        if room.inner().game_phase != 2 || room.inner().game_ended {
            return Err(StratepigError::with("room not in correct state"));
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "Must be in a game (not in placement or endgame state)"
    }
}
