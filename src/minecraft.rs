use crate::player::PlayerStats;
use std::error::Error;
use log::{debug, warn, error};
use rcon::Connection;
use serde::Deserialize;
use std::cell::RefCell;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct MinecraftServer {
    pub name: String,
    rcon_address: String,
    rcon_password: String,
    data_path: PathBuf,
    #[serde(skip)]
    rcon_connection: RefCell<Option<Connection>>,
}

#[derive(Debug)]
pub struct ServerStats {
    banlist_len: u32,
    ip_banlist_len: u32,
    whitelist_len: u32,
    connected_players: u32,
}

impl MinecraftServer {
    pub async fn stats(&self) -> Option<ServerStats> {
        debug!("Getting stats for server '{}'...", &self.name);
        Some(ServerStats {
            banlist_len: {
                debug!("Querying '{}' server banlist with RCON...", &self.name);
                match self.rcon_command("banlist players").await {
                    Ok(raw) => {
                        if &raw[..12] == "There are no" {
                            0
                        } else {
                            raw.lines().count() as u32 - 1
                        }
                    },
                    Err(e) => {
                        error!("Couldn't get server '{}' banlist: {}", &self.name, e);
                        return None;
                    }
                }
            },
            ip_banlist_len: {
                debug!("Querying '{}' server banlist with RCON...", &self.name);
                match self.rcon_command("banlist players").await {
                    Ok(raw) => {
                        if &raw[..12] == "There are no" {
                            0
                        } else {
                            raw.lines().count() as u32 - 1
                        }
                    },
                    Err(e) => {
                        error!("Couldn't get server '{}' IP banlist: {}", &self.name, e);
                        return None;
                    },
                }
            },
            whitelist_len: {
                debug!("Querying '{}' server whitelist with RCON...", &self.name);
                match self.rcon_command("whitelist list").await {
                    Ok(raw) => {
                        if &raw[..12] == "There are no" {
                            0
                        } else {
                            let list_raw = raw.split(": ").nth(1).unwrap();
                            list_raw.split(", ").count() as u32
                        }
                    },
                    Err(e) => {
                        error!("Couldn't get server '{}' whitelist: {}", &self.name, e);
                        return None;
                    },
                }
            },
            connected_players: {
                debug!("Querying '{}' server player list with RCON...", &self.name);
                match self.rcon_command("list").await {
                    Ok(raw) => raw.split(" ").nth(2).unwrap().parse().unwrap(),
                    Err(e) => {
                        error!("Couldn't get server '{}' connected players: {}", &self.name, e);
                        return None;
                    },
                }
            }
        })
    }

    // TODO: rcon functionality behind feature gate?
    // TODO: try recreating connection if it's cached and command fails, before failing
    async fn rcon_command(&self, cmd: &str) -> Result<String, Box<dyn Error>> {
        let mut rcon_opt = self.rcon_connection.borrow_mut();
        if rcon_opt.is_some() {
            debug!("Cached RCON connection found for server '{}'", &self.name);
        } else {
            let conn = Connection::builder()
                .enable_minecraft_quirks(true)
                .connect(&self.rcon_address, &self.rcon_password)
                .await?;
            debug!("Starting RCON connection for server '{}'", &self.name);
            rcon_opt.replace(conn);
        }

        match rcon_opt
            .as_mut()
            .unwrap()
            .cmd(cmd)
            .await
        {
            Ok(r) => Ok(r.clone()),
            Err(e) => {
                warn!(
                    "FAILED running command '{}' on server '{}': {}, retrying once...",
                    cmd, self.name, e
                );
                let conn = Connection::builder()
                    .enable_minecraft_quirks(true)
                    .connect(&self.rcon_address, &self.rcon_password)
                    .await?;
                debug!("Starting RCON connection for server '{}'", &self.name);
                rcon_opt.replace(conn);

                Ok(rcon_opt.as_mut().unwrap().cmd(cmd).await?)
            }
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
                let uuid = filepath.file_name().unwrap();
                let adv_path = self.data_path.join("world/advancements").join(uuid);
                let player_stats = match PlayerStats::from_stats_files(&filepath, &adv_path).await {
                    Ok(p) => {
                        debug!("Got stats/advancements for player {}", &p.username);
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
