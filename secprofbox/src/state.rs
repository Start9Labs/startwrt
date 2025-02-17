pub struct WifiConnection {
    pub interface: String,
    pub mac: String,
    pub key_id: Option<String>,
    pub ip: Option<String>,
}

pub struct State {
    pub wifi_connections: Vec<WifiConnection>,
}
