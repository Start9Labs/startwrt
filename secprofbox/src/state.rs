use serde::Deserialize;

use crate::watchutil::Watch;
use std::{collections::HashMap, net::Ipv4Addr, net::Ipv6Addr};

#[derive(Debug, Default)]
pub struct Connection {
    pub key_id: Option<String>,
    pub profile: Option<String>,
    pub ipv4: Option<Ipv4Addr>,
    pub ipv6: Option<Ipv6Addr>,
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct ConnectionId {
    pub interface: String,
    pub mac: String,
}

#[derive(Debug, Default)]
pub struct State {
    pub connections: HashMap<ConnectionId, Connection>,
    pub config: Config,
}

pub type WatchState = Watch<State>;

#[derive(Deserialize, Debug)]
pub enum LanAccess {
    AllDevices,
    NoDevices,
    OtherProfile(Vec<String>),
}

#[derive(Deserialize, Debug)]
pub struct SecProfile {
    pub lan: LanAccess,
    pub wan: bool,
}

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    pub keyid_to_profile: HashMap<String, String>,
    pub profiles: HashMap<String, SecProfile>,
}

pub fn set_config(state: &WatchState, config: Config) {
    state.send_modify(|state| {
        state.config = config;
        for conn in state.connections.values_mut() {
            conn.profile = conn
                .key_id
                .as_ref()
                .and_then(|k| state.config.keyid_to_profile.get(k))
                .cloned();
        }
    });
}
