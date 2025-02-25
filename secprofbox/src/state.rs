use crate::watchutil::Watch;
use std::{collections::HashMap, net::Ipv4Addr, net::Ipv6Addr};

#[derive(Debug, Default)]
pub struct Connection {
    pub key_id: Option<String>,
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
}

pub type WatchState = Watch<State>;
