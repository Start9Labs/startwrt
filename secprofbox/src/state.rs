use crate::watchutil::Watch;
use color_eyre::eyre::{bail, Context, Error};
use macaddr::MacAddr;
use serde::Deserialize;
use std::collections::{BTreeSet, HashMap};
use std::net::IpAddr;
use tokio::signal::unix::{signal, SignalKind};
use tracing::info;

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
            .and_then(|k| Some(config.keyids.get(k)?.profile.clone()))
            .or_else(|| config.interface_to_profile.get(&id.interface).cloned())
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

#[derive(Deserialize, Debug)]
pub struct KeyId {
    pub profile: String,
    pub password: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    pub interface_to_profile: HashMap<String, String>,
    pub keyids: HashMap<String, KeyId>,
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

pub const CONFIG_PATH: &str = "/etc/config/secprof";

pub fn load_config() -> Result<Config, Error> {
    use uciedit::{parse_config, UciSection};

    #[derive(UciSection)]
    #[uci(ty = "profile")]
    pub struct UciSecProfile {
        lan_acces: u8,
        wan_acces: u8,
        lan_whitelist: Option<String>,
    }

    #[derive(UciSection)]
    #[uci(ty = "wpapassword")]
    pub struct UciWpaPassword {
        password: String,
        profile: String,
    }

    let mut config = Config::default();
    // TODO: use read_sections instead of rewrite_sections (once implemented)
    parse_config(CONFIG_PATH, |mut ctx| {
        while ctx.step() {
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
            if let Ok(UciWpaPassword { password, profile }) = ctx.get() {
                let Some(name) = ctx.name() else {
                    bail!("all wpa passwords must be named")
                };
                let name = name.into_owned();
                config
                    .keyids
                    .insert(name.clone(), KeyId { profile, password });
            }
        }
        Ok(())
    })
    .with_context(|| format!("loading config {CONFIG_PATH}"))?;
    Ok(config)
}

pub async fn reload_config_sighup(state: WatchState) -> Result<(), Error> {
    let mut stream = signal(SignalKind::hangup())?;

    loop {
        stream.recv().await;
        info!("reloading {CONFIG_PATH}");
        state.send_modify(|state| {
            state.config = load_config()?;
            for (id, con) in &mut state.connections {
                con.update_profile(id, &state.config);
            }
            Ok::<_, Error>(())
        })?;
    }
}
