use crate::minecraft::MinecraftServer;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Configuration {
    listen_port: u16,
    servers: Vec<MinecraftServer>,
}
