use crate::watchutil::Watch;
use color_eyre::eyre::{bail, Error};
use macaddr::MacAddr;
use serde::Deserialize;
use std::collections::{BTreeSet, HashMap};
use std::net::IpAddr;

#[derive(Debug, Default)]
pub struct Connection {
    pub key_id: Option<String>,
    pub profile: Option<String>,
    pub ips: BTreeSet<IpAddr>,
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

use uciedit::{rewrite_sections, UciSection};

#[derive(UciSection)]
#[uci(ty = "profile")]
pub struct UciSecProfile {
    lan_acces: u8,
    wan_acces: u8,
    lan_whitelist: Option<String>,
}

#[derive(UciSection, Debug, Clone)]
#[uci(ty = "wpapassword")]
pub struct WpaPassword {
    password: String,
    profile: String,
}

pub fn load_config() -> Result<Config, Error> {
    let mut config = Config::default();
    // TODO: use read_sections instead of rewrite_sections (once implemented)
    rewrite_sections("/etc/config/secprof", |ctx| {
        if let Ok(UciSecProfile {
            lan_acces,
            wan_acces,
            lan_whitelist,
        }) = ctx.get()
        {
            let Some(name) = ctx.name() else {
                bail!("all security profiles must be named")
            };
            config.profiles.insert(
                name.into_owned(),
                SecProfile {
                    lan: if lan_acces > 0 {
                        LanAccess::AllDevices
                    } else if let Some(whitelist) = lan_whitelist {
                        LanAccess::OtherProfile(
                            whitelist.split(',').map(|s| s.to_owned()).collect(),
                        )
                    } else {
                        LanAccess::NoDevices
                    },
                    wan: wan_acces > 0,
                },
            );
        }
        if let Ok(WpaPassword { password, profile }) = ctx.get() {
            let Some(name) = ctx.name() else {
                bail!("all wpa passwords must be named")
            };
            let name = name.into_owned();
            config.keyid_to_profile.insert(name.clone(), profile);
            // TODO: save the password and reload the wpa service
        }
        Ok(())
    })?;
    Ok(config)
}
