#[macro_use]
extern crate env_logger;
#[macro_use]
extern crate lazy_static;

use log::{debug, info};

mod config;
mod minecraft;
mod player;
mod utils;

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Starting glass-pane...");

    let config_file_loc = match std::env::args().nth(1) {
        Some(loc) => loc,
        None => panic!("No config file location given"),
    };

    debug!("Loading configuration from {}...", &config_file_loc);
    let config: config::Configuration = serde_yaml::from_str(
        &std::fs::read_to_string(&config_file_loc).unwrap_or_else(|e| {
            panic!(
                "Couldn't read config file at {}, error: {}",
                config_file_loc, e
            )
        }),
    )
    .unwrap();
    debug!(
        "Configuration successfully loaded from {}",
        &config_file_loc
    );

    for server in config.servers.iter() {
        println!(
            "server: {}, whitelist length: {:?}, banlist length: {:?}, ip banlist length: {:?}\nstats: {:?}",
            &server.name,
            server.whitelist_len().await,
            server.banlist_len().await,
            server.ip_banlist_len().await,
            server.get_player_stats().await,
        );
    }
}
