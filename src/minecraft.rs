use crate::player::PlayerStats;
use log::{debug, error};
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
    banlist_len: Option<u32>,
    ip_banlist_len: Option<u32>,
    whitelist_len: Option<u32>,
}

impl MinecraftServer {
    pub async fn stats(&self) -> ServerStats {
        debug!("Getting stats for server '{}'...", &self.name);
        ServerStats {
            banlist_len: self.banlist_len().await,
            ip_banlist_len: self.ip_banlist_len().await,
            whitelist_len: self.whitelist_len().await,
        }
    }

    // TODO: rcon functionality behind feature gate?
    // TODO: try recreating connection if it's cached and command fails, before failing
    async fn rcon_command(&self, cmd: &str) -> Option<String> {
        let mut rcon_opt = self.rcon_connection.borrow_mut();
        if rcon_opt.is_some() {
            debug!("Open RCON connection found for server '{}'", &self.name);
        } else {
            match Connection::builder()
                .enable_minecraft_quirks(true)
                .connect(&self.rcon_address, &self.rcon_password)
                .await
            {
                Ok(c) => {
                    debug!("Starting RCON connection for server '{}'", &self.name);
                    rcon_opt.replace(c);
                    ()
                }
                Err(e) => {
                    error!(
                        "FAILED starting RCON connection to server '{}': {}",
                        self.name, e
                    );
                    return None;
                }
            }
        }

        match &self
            .rcon_connection
            .borrow_mut()
            .as_mut()
            .unwrap()
            .cmd(cmd)
            .await
        {
            Ok(r) => Some(r.clone()),
            Err(e) => {
                error!(
                    "FAILED running command '{}' on server '{}': {}",
                    cmd, self.name, e
                );
                None
            }
        }
    }

    async fn whitelist_len(&self) -> Option<u32> {
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

    async fn ip_banlist_len(&self) -> Option<u32> {
        debug!("Querying '{}' server IP banlist with RCON...", &self.name);
        self.rcon_command("banlist ips").await.map(|raw| {
            if &raw[..12] == "There are no" {
                0
            } else {
                raw.lines().count() as u32 - 1
            }
        })
    }

    async fn banlist_len(&self) -> Option<u32> {
        debug!("Querying '{}' server banlist with RCON...", &self.name);
        self.rcon_command("banlist players").await.map(|raw| {
            if &raw[..12] == "There are no" {
                0
            } else {
                raw.lines().count() as u32 - 1
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
