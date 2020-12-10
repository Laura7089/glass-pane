use crate::player::PlayerStats;
use log::{debug, error};
use rcon::Connection;
use serde::Deserialize;
use std::error::Error;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
pub struct MinecraftServer {
    pub name: String,
    rcon_address: String,
    rcon_password: String,
    data_path: PathBuf,
}

impl MinecraftServer {
    // TODO: rcon functionality behind feature gate?
    // TODO: cache connections?
    async fn rcon_command(&self, cmd: &str) -> Option<String> {
        let mut conn = match Connection::builder()
            .enable_minecraft_quirks(true)
            .connect(&self.rcon_address, &self.rcon_password)
            .await
        {
            Ok(c) => c,
            Err(e) => {
                error!(
                    "FAILED starting RCON connection to server '{}': {}",
                    self.name, e
                );
                return None;
            }
        };
        match conn.cmd(cmd).await {
            Ok(r) => Some(r),
            Err(e) => {
                error!(
                    "FAILED running command '{}' on server '{}': {}",
                    cmd, self.name, e
                );
                None
            }
        }
    }

    pub async fn whitelist_len(&self) -> Option<u32> {
        debug!("Querying '{}' server whitelist with RCON...", &self.name);
        self.rcon_command("whitelist list").await.map(|raw| {
            if &raw[..12] == "There are no" {
                0
            } else {
                let list_raw = raw.split(": ").nth(1).unwrap();
                list_raw.split(", ").count() as u32
            }
        })
    }

    pub async fn ip_banlist_len(&self) -> Option<u32> {
        debug!("Querying '{}' server IP banlist with RCON...", &self.name);
        self.rcon_command("banlist ips").await.map(|raw| {
            if &raw[..12] == "There are no" {
                0
            } else {
                let list_raw = raw.split(": ").nth(1).unwrap();
                list_raw.split(", ").count() as u32
            }
        })
    }

    pub async fn banlist_len(&self) -> Option<u32> {
        debug!("Querying '{}' server banlist with RCON...", &self.name);
        self.rcon_command("banlist players").await.map(|raw| {
            if &raw[..12] == "There are no" {
                0
            } else {
                let list_raw = raw.split(": ").nth(1).unwrap();
                list_raw.split(", ").count() as u32
            }
        })
    }

    pub async fn get_player_stats(&self) -> Vec<PlayerStats> {
        let mut ret = Vec::new();
        let stats_path = self.data_path.join("world/stats");
        debug!(
            "Getting player stats for server '{}' from '{}'...",
            &self.name,
            stats_path.to_str().unwrap()
        );

        let stats_dir = match std::fs::read_dir(&stats_path) {
            Ok(d) => d,
            Err(e) => {
                error!(
                    "Couldn't get stats from directory '{}', error: {}",
                    stats_path.to_str().unwrap(),
                    e
                );
                return ret;
            }
        };
        for file in stats_dir {
            let filepath = file.unwrap().path();
            if filepath.extension() == Some(std::ffi::OsStr::new("json")) {
                let uuid = filepath.file_name().unwrap();
                let adv_path =
                    PathBuf::from("world/advancements").join(uuid);
                let player_stats = match PlayerStats::from_stats_files(&filepath, &adv_path).await {
                    Ok(p) => {
                        debug!("Got stats/advancements for player {}", &p.id.username);
                        p
                    }
                    Err(e) => {
                        error!(
                            "Couldn't read player stats/advancements for '{}', error: {}",
                            uuid.to_str().unwrap(),
                            e
                        );
                        continue;
                    }
                };
                ret.push(player_stats);
            }
        }
        ret
    }
}
