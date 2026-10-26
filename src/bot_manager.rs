use std::collections::HashMap;
use std::net::ToSocketAddrs;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;

use crate::bot::Socks5Config;
use crate::bot_state::{BotCommand, BotState, BotStatus, CmdSender};
use crate::events::{WsEvent, WsTx};
use crate::items::ItemsDat;
use crate::options::{self, SavedBot, SavedBotAuth, SavedProxy};

pub struct BotEntry {
    pub username:  String,
    pub stop_flag: Arc<AtomicBool>,
    pub state:     Arc<RwLock<BotState>>,
    pub cmd_tx:    CmdSender,
    saved:         SavedBot,
    persisted:     bool,
}

pub struct BotManager {
    next_id:       u32,
    pub bots:      HashMap<u32, BotEntry>,
    pub items_dat: Arc<ItemsDat>,
    pub ws_tx:     WsTx,
}

#[derive(serde::Serialize)]
pub struct BotInfo {
    pub id:       u32,
    pub username: String,
    pub status:  String,
    pub world:   String,
    pub pos_x:   f32,
    pub pos_y:   f32,
    pub gems:    i32,
    pub ping_ms: u32,
}

fn socks_from_saved(proxy: Option<&SavedProxy>) -> Option<Socks5Config> {
    let p = proxy?;
    let addr = format!("{}:{}", p.host, p.port)
        .parse()
        .ok()
        .or_else(|| {
            format!("{}:{}", p.host, p.port)
                .to_socket_addrs()
                .ok()
                .and_then(|mut addrs| addrs.next())
        })?;
    Some(Socks5Config {
        proxy_addr: addr,
        username: p.username.clone(),
        password: p.password.clone(),
    })
}

fn saved_from_socks(proxy: &Socks5Config) -> SavedProxy {
    SavedProxy {
        host: proxy.proxy_addr.ip().to_string(),
        port: proxy.proxy_addr.port(),
        username: proxy.username.clone(),
        password: proxy.password.clone(),
    }
}

impl BotManager {
    pub fn new(ws_tx: WsTx) -> Self {
        Self { next_id: 0, bots: HashMap::new(), items_dat: Arc::new(ItemsDat::load()), ws_tx }
    }

    pub fn spawn(
        &mut self,
        username: String,
        password: String,
        proxy: Option<Socks5Config>,
        proxy_saved: Option<SavedProxy>,
    ) -> u32 {
        let saved = SavedBot {
            key: options::persist_key_legacy(&username),
            username: username.clone(),
            auth: SavedBotAuth::Legacy {
                username: username.clone(),
                password: password.clone(),
            },
            proxy: proxy_saved.or_else(|| proxy.as_ref().map(saved_from_socks)),
        };
        self.spawn_legacy_inner(username, password, proxy, saved)
    }

    pub fn spawn_ltoken(
        &mut self,
        ltoken_str: String,
        proxy: Option<Socks5Config>,
        proxy_saved: Option<SavedProxy>,
    ) -> u32 {
        let saved = SavedBot {
            key: options::persist_key_ltoken(&ltoken_str),
            username: String::new(),
            auth: SavedBotAuth::Ltoken {
                ltoken: ltoken_str.clone(),
            },
            proxy: proxy_saved.or_else(|| proxy.as_ref().map(saved_from_socks)),
        };
        self.spawn_ltoken_inner(ltoken_str, proxy, saved)
    }

    fn spawn_legacy_inner(
        &mut self,
        username: String,
        password: String,
        proxy: Option<Socks5Config>,
        saved: SavedBot,
    ) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        let stop_flag   = Arc::new(AtomicBool::new(false));
        let stop_clone  = stop_flag.clone();
        let uname       = username.clone();
        let pass        = password.clone();

        let state = Arc::new(RwLock::new(BotState {
            status: BotStatus::Connecting,
            ..Default::default()
        }));
        let state_clone = state.clone();

        let (cmd_tx, cmd_rx) = mpsc::channel::<BotCommand>();

        let items_dat = self.items_dat.clone();
        let ws_tx_clone = self.ws_tx.clone();

        std::thread::spawn(move || {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut bot = crate::bot::Bot::new(&uname, &pass, proxy, state_clone, cmd_rx, items_dat, id, Some(ws_tx_clone));
                bot.run(stop_clone);
            })) {
                Ok(_)  => println!("[Bot:{id}] Stopped."),
                Err(_) => println!("[Bot:{id}] Crashed."),
            }
        });

        self.bots.insert(id, BotEntry {
            username: username.clone(),
            stop_flag,
            state,
            cmd_tx,
            saved,
            persisted: false,
        });
        let _ = self.ws_tx.send(WsEvent::BotAdded { bot_id: id, username });
        id
    }

    fn spawn_ltoken_inner(
        &mut self,
        ltoken_str: String,
        proxy: Option<Socks5Config>,
        saved: SavedBot,
    ) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        let stop_flag  = Arc::new(AtomicBool::new(false));
        let stop_clone = stop_flag.clone();

        let state = Arc::new(RwLock::new(BotState {
            status: BotStatus::Connecting,
            ..Default::default()
        }));
        let state_clone = state.clone();

        let (cmd_tx, cmd_rx) = mpsc::channel::<BotCommand>();

        let items_dat = self.items_dat.clone();
        let ws_tx_clone = self.ws_tx.clone();

        std::thread::spawn(move || {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut bot = crate::bot::Bot::new_ltoken(&ltoken_str, proxy, state_clone, cmd_rx, items_dat, id, Some(ws_tx_clone));
                bot.run(stop_clone);
            })) {
                Ok(_)  => println!("[Bot:{id}] Stopped."),
                Err(_) => println!("[Bot:{id}] Crashed."),
            }
        });

        let username = saved.username.clone();
        self.bots.insert(id, BotEntry {
            username: username.clone(),
            stop_flag,
            state,
            cmd_tx,
            saved,
            persisted: false,
        });
        let _ = self.ws_tx.send(WsEvent::BotAdded { bot_id: id, username });
        id
    }

    pub fn restore_saved(&mut self) {
        let saved = options::load_saved_bots();
        if saved.is_empty() {
            return;
        }
        println!("[Hub] restoring {} saved bot(s)", saved.len());
        for bot in saved {
            let proxy = socks_from_saved(bot.proxy.as_ref());
            match bot.auth.clone() {
                SavedBotAuth::Legacy { username, password } => {
                    self.spawn_legacy_inner(username, password, proxy, bot);
                }
                SavedBotAuth::Ltoken { ltoken } => {
                    self.spawn_ltoken_inner(ltoken, proxy, bot);
                }
            }
        }
    }

    pub fn persist_success(&mut self, id: u32) {
        let Some(entry) = self.bots.get_mut(&id) else { return };
        if entry.persisted {
            return;
        }
        let live_name = {
            let s = entry.state.read().unwrap();
            if !s.username.is_empty() {
                s.username.clone()
            } else {
                entry.username.clone()
            }
        };
        if !live_name.is_empty() {
            entry.username = live_name.clone();
            entry.saved.username = live_name;
        }
        let saved = entry.saved.clone();
        entry.persisted = true;
        if let Err(e) = options::upsert_saved_bot(saved) {
            println!("[Hub] failed to save bot: {e}");
        }
    }

    pub fn set_username(&mut self, id: u32, username: String) {
        let Some(entry) = self.bots.get_mut(&id) else { return };
        entry.username = username.clone();
        entry.saved.username = username;
        if entry.persisted {
            let saved = entry.saved.clone();
            let _ = options::upsert_saved_bot(saved);
        }
    }

    pub fn stop(&mut self, id: u32) -> bool {
        if let Some(entry) = self.bots.remove(&id) {
            let key = entry.saved.key.clone();
            entry.stop_flag.store(true, Ordering::Relaxed);
            let _ = self.ws_tx.send(WsEvent::BotRemoved { bot_id: id });
            if let Err(e) = options::remove_saved_bot(&key) {
                println!("[Hub] failed to forget bot: {e}");
            }
            true
        } else {
            false
        }
    }

    pub fn list(&self) -> Vec<BotInfo> {
        self.bots.iter().map(|(id, e)| {
            let s = e.state.read().unwrap();
            let username = if !s.username.is_empty() {
                s.username.clone()
            } else {
                e.username.clone()
            };
            BotInfo {
                id:       *id,
                username,
                status:  s.status.to_string(),
                world:   s.world_name.clone(),
                pos_x:   s.pos_x,
                pos_y:   s.pos_y,
                gems:    s.gems,
                ping_ms: s.ping_ms,
            }
        }).collect()
    }

    pub fn get_state(&self, id: u32) -> Option<BotState> {
        self.bots.get(&id).map(|e| e.state.read().unwrap().clone())
    }

    pub fn send_cmd(&self, id: u32, cmd: BotCommand) -> bool {
        self.bots.get(&id).map(|e| e.cmd_tx.send(cmd).is_ok()).unwrap_or(false)
    }

    pub fn run_script(&self, id: u32, content: String) -> bool {
        self.send_cmd(id, BotCommand::RunScript { content })
    }

    pub fn find_by_name(&self, name: &str) -> Option<(Arc<RwLock<BotState>>, CmdSender)> {
        self.bots.values()
            .find(|e| e.username.eq_ignore_ascii_case(name))
            .map(|e| (e.state.clone(), e.cmd_tx.clone()))
    }

    pub fn stop_by_name(&mut self, name: &str) -> bool {
        if let Some(id) = self.bots.iter()
            .find(|(_, e)| e.username.eq_ignore_ascii_case(name))
            .map(|(id, _)| *id)
        {
            self.stop(id)
        } else {
            false
        }
    }
}
