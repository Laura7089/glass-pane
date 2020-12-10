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

const DISTANCE_KEYS: [&'static str; 9] = [
    "minecraft:climb_one_cm",
    "minecraft:crouch_one_cm",
    "minecraft:fall_one_cm",
    "minecraft:fly_one_cm",
    "minecraft:sprint_one_cm",
    "minecraft:swim_one_cm",
    "minecraft:walk_one_cm",
    "minecraft:walk_on_water_one_cm",
    "minecraft:walk_under_water_one_cm",
];

#[derive(Debug)]
pub struct PlayerStats {
    pub username: String,
    pub uuid: Uuid,
    pub blocks_mined: u32,
    pub items_picked_up: u32,
    pub items_used: u32,
    pub items_crafted: u32,
    // TODO: mob types?
    pub mobs_killed: u32,
    pub deaths: u32,
    pub jumps: u32,
    pub minutes_played: u32,
    pub damage_taken: u32,
    pub damage_dealt: u32,
    pub adv_made: u32,
    pub cm_travelled: u32,
}

#[derive(Deserialize, Debug)]
struct PlayerStatsFull {
    #[serde(default, rename = "minecraft:mined")]
    blocks_mined: HashMap<String, u32>,
    #[serde(default, rename = "minecraft:picked_up")]
    items_picked_up: HashMap<String, u32>,
    #[serde(default, rename = "minecraft:used")]
    items_used: HashMap<String, u32>,
    #[serde(default, rename = "minecraft:crafted")]
    items_crafted: HashMap<String, u32>,
    #[serde(default, rename = "minecraft:custom")]
    misc_stats: HashMap<String, u32>,
}

impl PlayerStatsFull {
    fn get_misc(&self, key: &str) -> u32 {
        *self.misc_stats.get(key).unwrap_or(&0)
    }
}

impl PlayerStats {
    pub async fn from_stats_files(
        stats_file: &Path,
        adv_file: &Path,
    ) -> Result<Self, Box<dyn Error>> {
        let file_raw = std::fs::read_to_string(stats_file)?;
        let stats_wrapped: serde_json::Value = serde_json::from_str(&file_raw)?;
        let stats_full: PlayerStatsFull =
            serde_json::value::from_value(stats_wrapped.get("stats").unwrap().clone())?;
        let uuid = Uuid::parse_str(stats_file.file_stem().unwrap().to_str().unwrap())?;

        let adv_made = std::fs::read_to_string(adv_file)?
            .lines()
            .filter(|l| l.contains("\"done\": true"))
            .count() as u32;

        // TODO: resolve unwraps
        Ok(Self {
            username: crate::utils::username_from_uuid(&uuid).await?,
            uuid,
            blocks_mined: stats_full.blocks_mined.values().sum(),
            items_picked_up: stats_full.items_picked_up.values().sum(),
            items_used: stats_full.items_used.values().sum(),
            items_crafted: stats_full.items_crafted.values().sum(),
            mobs_killed: stats_full.get_misc("minecraft:mob_kills"),
            damage_dealt: stats_full.get_misc("minecraft:damage_dealt"),
            damage_taken: stats_full.get_misc("minecraft:damage_taken"),
            deaths: stats_full.get_misc("minecraft:deaths"),
            minutes_played: stats_full.get_misc("minecraft:play_one_minute"),
            jumps: stats_full.get_misc("minecraft:jump"),
            cm_travelled: DISTANCE_KEYS.iter().map(|k| stats_full.get_misc(k)).sum(),
            adv_made,
        })
    }
}
