use log::debug;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::sync::Mutex;
use uuid::Uuid;

// TODO: do I need an Arc?
lazy_static! {
    static ref USERNAME_CACHE: Mutex<HashMap<Uuid, String>> = Mutex::new(HashMap::new());
}

#[derive(Debug)]
pub struct PlayerStats {
    pub id: PlayerId,
    pub blocks_mined: u32,
    pub items_picked_up: u32,
    pub items_used: u32,
    pub items_crafted: u32,
    // TODO: mob types?
    pub mobs_killed: u32,
}

#[derive(Debug, Default)]
pub struct PlayerId {
    pub username: String,
    pub uuid: Uuid,
}

#[derive(Deserialize)]
struct StatsWrapped {
    stats: PlayerStatsFull,
}

#[derive(Deserialize, Debug)]
struct PlayerStatsFull {
    #[serde(default, rename = "minecraft:mined")]
    pub blocks_mined: HashMap<String, u32>,
    #[serde(default, rename = "minecraft:picked_up")]
    pub items_picked_up: HashMap<String, u32>,
    #[serde(default, rename = "minecraft:used")]
    pub items_used: HashMap<String, u32>,
    #[serde(default, rename = "minecraft:crafted")]
    pub items_crafted: HashMap<String, u32>,
    #[serde(default, rename = "minecraft:killed")]
    pub mobs_killed: HashMap<String, u32>,
    #[serde(default, rename = "minecraft:custom")]
    pub misc_stats: HashMap<String, u32>,
}

impl PlayerStats {
    pub async fn from_stats_file(file: &Path) -> Result<Self, Box<dyn Error>> {
        let file_raw = std::fs::read_to_string(file)?;
        let wrapped: StatsWrapped = serde_yaml::from_str(&file_raw)?;
        let stats_full: PlayerStatsFull = wrapped.stats;
        let uuid = Uuid::parse_str(file.file_stem().unwrap().to_str().unwrap())?;

        // TODO: resolve unwraps
        Ok(Self {
            id: PlayerId {
                username: crate::utils::username_from_uuid(&uuid).await?,
                uuid,
            },
            blocks_mined: stats_full.blocks_mined.values().sum(),
            items_picked_up: stats_full.items_picked_up.values().sum(),
            items_used: stats_full.items_used.values().sum(),
            items_crafted: stats_full.items_crafted.values().sum(),
            mobs_killed: stats_full.mobs_killed.values().sum(),
        })
    }
}
