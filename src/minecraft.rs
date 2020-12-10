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
    async fn get_connection(&self) -> Result<Connection, Box<dyn Error>> {
        Ok(Connection::builder()
            .enable_minecraft_quirks(true)
            .connect(&self.rcon_address, &self.rcon_password)
            .await?)
    }

    pub async fn whitelist_len(&self) -> Result<u32, Box<dyn Error>> {
        debug!("Querying '{}' server whitelist with RCON...", &self.name);
        // TODO: cache connections?
        let server_response = self.get_connection().await?.cmd("whitelist list").await?;
        if &server_response[..12] == "There are no" {
            Ok(0)
        } else {
            let list_raw = server_response.split(": ").nth(1).unwrap();
            Ok(list_raw.split(", ").count() as u32)
        }
    }

    pub async fn ip_banlist_len(&self) -> Result<u32, Box<dyn Error>> {
        debug!("Querying '{}' server IP banlist with RCON...", &self.name);
        let server_response = self.get_connection().await?.cmd("banlist ips").await?;
        if &server_response[..12] == "There are no" {
            Ok(0)
        } else {
            Ok(server_response.lines().count() as u32 - 1)
        }
    }

    pub async fn banlist_len(&self) -> Result<u32, Box<dyn Error>> {
        debug!("Querying '{}' server banlist with RCON...", &self.name);
        let server_response = self.get_connection().await?.cmd("banlist players").await?;
        if &server_response[..12] == "There are no" {
            Ok(0)
        } else {
            Ok(server_response.lines().count() as u32 - 1)
        }
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
                let adv_path = PathBuf::from("world/advancements").join(filepath.file_name().unwrap());
                let player_stats = match PlayerStats::from_stats_files(&filepath, &adv_path).await {
                    Ok(p) => {
                        debug!("Got stats for player {}", &p.id.username);
                        p
                    }
                    Err(e) => {
                        error!(
                            "Couldn't read player stats file '{}', error: {}",
                            &filepath.to_str().unwrap(),
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
