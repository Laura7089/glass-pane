use crate::player::{PlayerId, PlayerStats};
use rcon::Connection;
use serde::Deserialize;
use std::error::Error;
use std::fmt;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct MinecraftServer {
    pub name: String,
    rcon_address: String,
    rcon_password: String,
    data_path: PathBuf,
    #[serde(skip)]
    connection: Option<Connection>,
}

impl fmt::Debug for MinecraftServer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MinecraftServer")
            .field("name", &self.name)
            .field("rcon_address", &self.rcon_address)
            .field("rcon_password", &self.rcon_password)
            .field("data_path", &self.data_path)
            .finish()
    }
}

impl MinecraftServer {
    pub fn new(
        name: String,
        rcon_address: String,
        rcon_password: String,
        data_path: PathBuf,
    ) -> Self {
        Self {
            name,
            rcon_address,
            rcon_password,
            connection: None,
            data_path,
        }
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        self.connection = Some(
            Connection::builder()
                .enable_minecraft_quirks(true)
                .connect(&self.rcon_address, &self.rcon_password)
                .await?,
        );
        Ok(())
    }

    pub fn disconnect(&mut self) {
        self.connection = None;
    }

    pub async fn command(&mut self, cmd: &str) -> Result<String, Box<dyn Error>> {
        Ok(self
            .connection
            .as_mut()
            .ok_or("Tried to run a command on a server without a connection".to_string())?
            .cmd(cmd)
            .await?)
    }

    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    pub async fn whitelist(&mut self) -> Result<Vec<PlayerId>, Box<dyn Error>> {
        let server_response = self.command("whitelist list").await?;
        unimplemented!()
    }

    pub async fn get_player_stats(&self) -> Result<Vec<PlayerStats>, Box<dyn Error>> {
        let mut ret = Vec::new();
        for file in std::fs::read_dir(self.data_path.join("world/stats"))? {
            let filepath = file?.path();
            if filepath.extension() == Some(std::ffi::OsStr::new("json")) {
                ret.push(PlayerStats::from_stats_file(&filepath)?);
            }
        }
        Ok(ret)
    }
}
