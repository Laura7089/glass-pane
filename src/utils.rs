use log::debug;
use serde::Deserialize;
use std::{
    collections::HashMap,
    error::Error,
    sync::{Arc, RwLock},
};
use uuid::Uuid;

// TODO: do I need an Arc?
lazy_static! {
    static ref USERNAME_CACHE: Arc<RwLock<HashMap<String, String>>> =
        Arc::new(RwLock::new(HashMap::new()));
    static ref UUID_CACHE: Arc<RwLock<HashMap<String, Uuid>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

#[derive(Deserialize)]
struct MojangPlayerName {
    name: String,
    #[serde(default)]
    changed_to_at: Option<u64>,
}

pub async fn username_from_uuid(uuid: &Uuid) -> Result<String, Box<dyn Error>> {
    let uuid = format!("{}", uuid.to_simple());
    // TODO: is unwrap okay here?
    let name_opt = USERNAME_CACHE
        .clone()
        .read()
        .unwrap()
        .get(&uuid)
        .map(|n| n.clone());
    if let Some(name) = name_opt {
        debug!("Name for UUID {} found in cache: {}", &uuid, &name);
        Ok(name)
    } else {
        debug!("Name for UUID {} not found, calling Mojang API...", &uuid);
        let name = serde_json::from_str::<Vec<MojangPlayerName>>(
            // TODO: use a static http client?
            &reqwest::get(&format!(
                "https://api.mojang.com/user/profiles/{}/names",
                uuid
            ))
            .await?
            .text()
            .await?,
        )?
        .into_iter()
        .max_by_key(|n| n.changed_to_at.ok_or(0))
        .unwrap()
        .name;
        // TODO: unwraps
        debug!("Name for UUID {} found with API: {}", &uuid, &name);
        USERNAME_CACHE
            .clone()
            .write()
            .unwrap()
            .insert(uuid, name.clone());
        Ok(name)
    }
}

// pub async fn _uuid_from_username(username: &str) -> Result<Uuid, Box<dyn Error>> {
//     let mut uuid_cache = UUID_CACHE.lock().unwrap();
//     if let Some(uuid) = uuid_cache.get(username) {
//         debug!("UUID for name '{}' found in cache: {}", username, &uuid);
//         Ok(*uuid)
//     } else {
//         debug!(
//             "UUID for name '{}' not found, calling Mojang API...",
//             username
//         );
//         let api_resp = serde_json::from_str::<HashMap<String, String>>(
//             &reqwest::get(&format!(
//                 "https://api.mojang.com/users/profiles/minecraft/{}",
//                 username
//             ))
//             .await?
//             .text()
//             .await?,
//         )?;
//         let uuid = Uuid::parse_str(&api_resp["id"])?;
//         debug!("UUID for name '{}' found with API: {}", username, &uuid);
//         uuid_cache.insert(username.to_string(), uuid);
//         Ok(uuid)
//     }
// }
