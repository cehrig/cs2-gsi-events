use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Provider {
    name: String,
    appid: usize,
    version: usize,
    steamid: String,
    timestamp: usize,
}
