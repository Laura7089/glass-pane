use rcon::Connection;
use serde::Deserialize;
use std::error::Error;
use std::path::PathBuf;
use std::fmt;

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
        f.debug_struct("MinecraftServer").field("name", &self.name).field("rcon_address", &self.rcon_address).field("rcon_password", &self.rcon_password).field("data_path", &self.data_path).finish()
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

    pub async fn command(&mut self, cmd: &str) -> Result<String, Box<dyn Error>> {
        Ok(self
            .connection
            .as_mut()
            .expect("Tried to run a command on a server without a connection")
            .cmd(cmd)
            .await?)
    }

    pub fn is_initialised(&self) -> bool {
        self.connection.is_some()
    }
}
