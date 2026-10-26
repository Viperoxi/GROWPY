use crate::bot_state::BotState;
use crate::inventory::{Inventory, InventoryItem};
use crate::items::ItemInfo;
use crate::protocol::variant::{Variant, VariantList};
use crate::protocol::packet::GameUpdatePacket;
use crate::player::Player;
use crate::world::{Tile, World, WorldNpc, WorldObject};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

pub(super) struct BotProxy {
    pub req_tx: crossbeam_channel::Sender<crate::script_channel::ScriptRequest>,
    pub reply_rx: crossbeam_channel::Receiver<crate::script_channel::ScriptReply>,
    pub state: Arc<RwLock<BotState>>,
    pub stop_flag: Arc<AtomicBool>,
}

impl BotProxy {
    pub fn request(
        &self,
        req: crate::script_channel::ScriptRequest,
    ) -> crate::script_channel::ScriptReply {
        if self.stop_flag.load(Ordering::Relaxed) {
            return crate::script_channel::ScriptReply::Ack;
        }
        if self.req_tx.send(req).is_err() {
            return crate::script_channel::ScriptReply::Ack;
        }
        loop {
            if self.stop_flag.load(Ordering::Relaxed) {
                return crate::script_channel::ScriptReply::Ack;
            }
            match self.reply_rx.recv_timeout(std::time::Duration::from_millis(50)) {
                Ok(reply) => return reply,
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {}
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                    return crate::script_channel::ScriptReply::Ack;
                }
            }
        }
    }
}

pub(super) struct LuaWorld {
    pub world: Arc<World>,
    pub players: Arc<Vec<Player>>,
    pub local_net_id: u32,
    pub local_user_id: u32,
    pub local_name: String,
    pub local_pos: (f32, f32),
}

pub(super) struct LuaInventory(pub Inventory);
pub(super) struct LuaInventoryItem(pub InventoryItem);
pub(super) struct LuaPlayer(pub Player);
pub(super) struct LuaTile(pub Tile);
pub(super) struct LuaNetObject(pub WorldObject);
pub(super) struct LuaItemInfo(pub ItemInfo);
pub(super) struct LuaGameUpdatePacket(pub GameUpdatePacket);
pub(super) struct LuaVariant(pub Variant);
pub(super) struct LuaVariantList(pub VariantList);
pub(super) struct LuaLogin {
    pub mac: String,
}
pub(super) struct LuaNpc(pub WorldNpc);
