use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use uuid::Uuid;

#[derive(Deserialize)]
struct PlayerStatsWrapped {
    stats: PlayerStats,
    #[serde(rename = "DataVersion")]
    data_version: String,
}

#[derive(Deserialize, Debug)]
pub struct PlayerStats {
    #[serde(skip)]
    id: PlayerId,
    #[serde(default, rename = "minecraft:mined")]
    blocks_mined: HashMap<String, u32>,
    #[serde(default, rename = "minecraft:picked_up")]
    items_picked_up: HashMap<String, u32>,
    #[serde(default, rename = "minecraft:used")]
    items_used: HashMap<String, u32>,
    #[serde(default, rename = "minecraft:crafted")]
    items_crafted: HashMap<String, u32>,
    #[serde(default, rename = "minecraft:killed")]
    mobs_killed: HashMap<String, u32>,
    #[serde(default, rename = "minecraft:custom")]
    misc_stats: HashMap<String, u32>,
}

#[derive(Debug, Default)]
pub struct PlayerId {
    username: String,
    uuid: Uuid,
}

impl PlayerStats {
    pub fn from_stats_file(file: &Path) -> Result<Self, Box<dyn Error>> {
        let file_raw = std::fs::read_to_string(file)?;
        let mut ret: PlayerStatsWrapped = serde_yaml::from_str(&file_raw)?;

        // TODO: resolve unwraps
        ret.stats.id = PlayerId::from_uuid_str(file.file_stem().unwrap().to_str().unwrap())?;
        Ok(ret.stats)
    }
}

impl PlayerId {
    pub fn from_uuid_str(uuid: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            username: "".into(),
            uuid: Uuid::parse_str(uuid)?,
        })
    }
}
