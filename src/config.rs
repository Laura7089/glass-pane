use crate::minecraft::MinecraftServer;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Configuration {
    pub listen_port: u16,
    pub servers: Vec<MinecraftServer>,
}
