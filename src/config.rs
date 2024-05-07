use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub db_path: String,
    pub jwt_key: String,
    pub api_base_path: String,
    pub listen_ip: String,
    pub listen_port: u16,
    pub max_release_size: u32,
    pub max_image_size: u32
}
