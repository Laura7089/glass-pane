#[macro_use]
extern crate env_logger;
use tokio::prelude::*;

mod config;
mod minecraft;

#[tokio::main]
async fn main() {
    let config_file_loc = match std::env::args().nth(1) {
        Some(loc) => loc,
        None => panic!("No config file location given"),
    };

    let config: config::Configuration = serde_yaml::from_str(
        &std::fs::read_to_string(&config_file_loc)
            .unwrap_or_else(|e| panic!("Couldn't read config file at {}, error: {}", config_file_loc, e)),
    )
    .unwrap();

    println!("{:?}", config);
}
