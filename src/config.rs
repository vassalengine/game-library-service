#[derive(Debug)]
pub struct Config {
    pub db_path: String,
    pub jwt_key: Vec<u8>,
    pub api_base_path: String,
    pub listen_ip: [u8; 4],
    pub listen_port: u16 
}
