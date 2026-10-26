use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::setup::cache_dir;

static LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn lock() -> std::sync::MutexGuard<'static, ()> {
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

fn options_path() -> PathBuf {
    cache_dir().join("options.json")
}

fn scripts_path() -> PathBuf {
    cache_dir().join("scripts.json")
}

fn bots_path() -> PathBuf {
    cache_dir().join("bots.json")
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalOptions {
    #[serde(default = "default_place_ms")]
    pub place_ms: u64,
    #[serde(default = "default_walk_ms")]
    pub walk_ms: u64,
    #[serde(default = "default_true")]
    pub auto_collect: bool,
    #[serde(default = "default_collect_range")]
    pub collect_range: u8,
    #[serde(default)]
    pub collect_path_check: bool,
    #[serde(default)]
    pub ignore_gems: bool,
    #[serde(default)]
    pub ignore_essences: bool,
    #[serde(default = "default_punch_reach")]
    pub punch_reach: u32,
    #[serde(default = "default_farm_holes")]
    pub farm_holes: u32,
    #[serde(default = "default_pickup_range")]
    pub pickup_range: u32,
}

fn default_place_ms() -> u64 {
    180
}
fn default_walk_ms() -> u64 {
    80
}
fn default_true() -> bool {
    true
}
fn default_collect_range() -> u8 {
    5
}
fn default_punch_reach() -> u32 {
    2
}
fn default_farm_holes() -> u32 {
    2
}
fn default_pickup_range() -> u32 {
    3
}

impl Default for GlobalOptions {
    fn default() -> Self {
        Self {
            place_ms: default_place_ms(),
            walk_ms: default_walk_ms(),
            auto_collect: true,
            collect_range: default_collect_range(),
            collect_path_check: false,
            ignore_gems: false,
            ignore_essences: false,
            punch_reach: default_punch_reach(),
            farm_holes: default_farm_holes(),
            pickup_range: default_pickup_range(),
        }
    }
}

impl GlobalOptions {
    pub fn sanitized(mut self) -> Self {
        self.collect_range = self.collect_range.clamp(1, 5);
        self.punch_reach = self.punch_reach.clamp(1, 4);
        self.farm_holes = self.farm_holes.clamp(1, 8);
        self.pickup_range = self.pickup_range.clamp(1, 8);
        self.place_ms = self.place_ms.min(10_000);
        self.walk_ms = self.walk_ms.min(10_000);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedScript {
    pub id: String,
    pub name: String,
    pub content: String,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ScriptLibrary {
    #[serde(default)]
    scripts: Vec<SavedScript>,
}

fn ensure_cache_dir() {
    let _ = fs::create_dir_all(cache_dir());
}

pub fn load_options() -> GlobalOptions {
    let _g = lock();
    load_options_unlocked()
}

fn load_options_unlocked() -> GlobalOptions {
    ensure_cache_dir();
    match fs::read_to_string(options_path()) {
        Ok(raw) => serde_json::from_str::<GlobalOptions>(&raw)
            .unwrap_or_default()
            .sanitized(),
        Err(_) => GlobalOptions::default(),
    }
}

pub fn save_options(opts: GlobalOptions) -> anyhow::Result<GlobalOptions> {
    let _g = lock();
    let opts = opts.sanitized();
    ensure_cache_dir();
    let data = serde_json::to_string_pretty(&opts)?;
    fs::write(options_path(), data)?;
    Ok(opts)
}

pub fn list_scripts() -> Vec<SavedScript> {
    let _g = lock();
    load_library_unlocked().scripts
}

pub fn resolve_script<'a>(scripts: &'a [SavedScript], query: &str) -> Result<&'a SavedScript, String> {
    let q = query.trim();
    if q.is_empty() {
        return Err("empty name".into());
    }
    if let Some(s) = scripts.iter().find(|s| s.id == q) {
        return Ok(s);
    }
    if let Some(s) = scripts.iter().find(|s| s.name == q) {
        return Ok(s);
    }
    let ci: Vec<&SavedScript> = scripts
        .iter()
        .filter(|s| s.name.eq_ignore_ascii_case(q))
        .collect();
    match ci.len() {
        0 => Err(format!("script not found: {q}")),
        1 => Ok(ci[0]),
        _ => Err(format!("several scripts named '{q}'")),
    }
}

pub fn find_script(query: &str) -> Result<SavedScript, String> {
    let scripts = list_scripts();
    resolve_script(&scripts, query).map(|s| s.clone())
}

fn load_library_unlocked() -> ScriptLibrary {
    ensure_cache_dir();
    match fs::read_to_string(scripts_path()) {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
        Err(_) => ScriptLibrary::default(),
    }
}

fn save_library_unlocked(lib: &ScriptLibrary) -> anyhow::Result<()> {
    ensure_cache_dir();
    let data = serde_json::to_string_pretty(lib)?;
    fs::write(scripts_path(), data)?;
    Ok(())
}

pub fn create_script(name: String, content: String) -> anyhow::Result<SavedScript> {
    let _g = lock();
    let mut lib = load_library_unlocked();
    let script = SavedScript {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.trim().to_string(),
        content,
        updated_at: now_secs(),
    };
    lib.scripts.push(script.clone());
    save_library_unlocked(&lib)?;
    Ok(script)
}

pub fn update_script(
    id: &str,
    name: Option<String>,
    content: Option<String>,
) -> anyhow::Result<Option<SavedScript>> {
    let _g = lock();
    let mut lib = load_library_unlocked();
    let Some(script) = lib.scripts.iter_mut().find(|s| s.id == id) else {
        return Ok(None);
    };
    if let Some(name) = name {
        script.name = name.trim().to_string();
    }
    if let Some(content) = content {
        script.content = content;
    }
    script.updated_at = now_secs();
    let out = script.clone();
    save_library_unlocked(&lib)?;
    Ok(Some(out))
}

pub fn delete_script(id: &str) -> anyhow::Result<bool> {
    let _g = lock();
    let mut lib = load_library_unlocked();
    let before = lib.scripts.len();
    lib.scripts.retain(|s| s.id != id);
    if lib.scripts.len() == before {
        return Ok(false);
    }
    save_library_unlocked(&lib)?;
    Ok(true)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedProxy {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum SavedBotAuth {
    Legacy {
        username: String,
        password: String,
    },
    Ltoken {
        ltoken: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedBot {
    pub key: String,
    #[serde(default)]
    pub username: String,
    pub auth: SavedBotAuth,
    #[serde(default)]
    pub proxy: Option<SavedProxy>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SavedBotsFile {
    #[serde(default)]
    bots: Vec<SavedBot>,
}

pub fn persist_key_legacy(username: &str) -> String {
    format!("legacy:{}", username.trim().to_lowercase())
}

pub fn persist_key_ltoken(ltoken: &str) -> String {
    let parts: Vec<&str> = ltoken.split('|').collect();
    if parts.len() >= 3 {
        format!("ltoken:{}:{}", parts[1], parts[2])
    } else {
        format!("ltoken:{}", ltoken.chars().take(32).collect::<String>())
    }
}

fn load_saved_bots_unlocked() -> SavedBotsFile {
    ensure_cache_dir();
    match fs::read_to_string(bots_path()) {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
        Err(_) => SavedBotsFile::default(),
    }
}

fn save_saved_bots_unlocked(file: &SavedBotsFile) -> anyhow::Result<()> {
    ensure_cache_dir();
    let data = serde_json::to_string_pretty(file)?;
    fs::write(bots_path(), data)?;
    Ok(())
}

pub fn load_saved_bots() -> Vec<SavedBot> {
    let _g = lock();
    load_saved_bots_unlocked().bots
}

pub fn upsert_saved_bot(bot: SavedBot) -> anyhow::Result<()> {
    let _g = lock();
    let mut file = load_saved_bots_unlocked();
    if let Some(existing) = file.bots.iter_mut().find(|b| b.key == bot.key) {
        *existing = bot;
    } else {
        file.bots.push(bot);
    }
    save_saved_bots_unlocked(&file)
}

pub fn remove_saved_bot(key: &str) -> anyhow::Result<bool> {
    let _g = lock();
    let mut file = load_saved_bots_unlocked();
    let before = file.bots.len();
    file.bots.retain(|b| b.key != key);
    if file.bots.len() == before {
        return Ok(false);
    }
    save_saved_bots_unlocked(&file)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::{persist_key_legacy, persist_key_ltoken, resolve_script, GlobalOptions, SavedScript};

    #[test]
    fn sanitized_clamps_ranges() {
        let opts = GlobalOptions {
            collect_range: 99,
            punch_reach: 0,
            farm_holes: 40,
            pickup_range: 0,
            place_ms: 50_000,
            ..Default::default()
        }
        .sanitized();
        assert_eq!(opts.collect_range, 5);
        assert_eq!(opts.punch_reach, 1);
        assert_eq!(opts.farm_holes, 8);
        assert_eq!(opts.pickup_range, 1);
        assert_eq!(opts.place_ms, 10_000);
    }

    #[test]
    fn persist_keys_are_stable() {
        assert_eq!(persist_key_legacy("Vaiprr"), persist_key_legacy("vaiprr"));
        assert_eq!(
            persist_key_ltoken("tokenA|rid1|mac1|wk1"),
            persist_key_ltoken("tokenB|rid1|mac1|wk2")
        );
        assert_ne!(
            persist_key_ltoken("tokenA|rid1|mac1|wk1"),
            persist_key_ltoken("tokenA|rid2|mac1|wk1")
        );
    }

    fn script(id: &str, name: &str) -> SavedScript {
        SavedScript {
            id: id.to_string(),
            name: name.to_string(),
            content: "--".into(),
            updated_at: 0,
        }
    }

    #[test]
    fn resolve_script_prefers_exact_name() {
        let scripts = vec![
            script("id-a", "Dirt farm"),
            script("id-b", "dirt farm"),
        ];
        let found = resolve_script(&scripts, "Dirt farm").unwrap();
        assert_eq!(found.id, "id-a");
    }

    #[test]
    fn resolve_script_case_insensitive_when_unique() {
        let scripts = vec![script("id-a", "Check mats")];
        let found = resolve_script(&scripts, "check mats").unwrap();
        assert_eq!(found.id, "id-a");
    }

    #[test]
    fn resolve_script_rejects_ambiguous_names() {
        let scripts = vec![
            script("id-a", "Farm"),
            script("id-b", "farm"),
        ];
        assert!(resolve_script(&scripts, "FARM").is_err());
    }
}
