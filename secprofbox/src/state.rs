use crate::watchutil::Watch;
use std::{collections::HashMap, net::IpAddr};

#[derive(Debug, Default)]
pub struct WifiConnection {
    pub key_id: Option<String>,
    pub ip: Option<IpAddr>,
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct WifiConnectionId {
    pub interface: String,
    pub mac: String,
}

#[derive(Debug, Default)]
pub struct State {
    pub wifi_connections: HashMap<WifiConnectionId, WifiConnection>,
}

pub type WatchState = Watch<State>;
