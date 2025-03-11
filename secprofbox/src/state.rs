use macaddr::MacAddr;
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

impl Connection {
    pub fn update_profile(&mut self, id: &ConnectionId, config: &Config) {
        self.profile = self
            .key_id
            .as_ref()
            .and_then(|k| config.keyid_to_profile.get(k))
            .or_else(|| config.interface_to_profile.get(&id.interface))
            .cloned();
    }
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct ConnectionId {
    pub interface: String,
    pub mac: MacAddr,
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
    pub interface_to_profile: HashMap<String, String>,
    pub keyid_to_profile: HashMap<String, String>,
    pub profiles: HashMap<String, SecProfile>,
}

pub fn set_config(state: &WatchState, config: Config) {
    state.send_modify(|state| {
        state.config = config;
        for (id, conn) in state.connections.iter_mut() {
            conn.update_profile(id, &state.config);
        }
    });
}
