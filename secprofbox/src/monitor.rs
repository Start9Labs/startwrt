use color_eyre::eyre::{bail, eyre, Context, Error};
use futures::try_join;
use inpt::split::Line;
use inpt::{inpt, Inpt};
use macaddr::MacAddr;
use std::net::IpAddr;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{error, info};

use crate::state::{ConnectionId, WatchState};
use crate::wpactrl::{Subscription, WpaCtrl};

// note the lazy .*? here, so that we deliniate on whitespace _unless_ otherwise forced to include it
#[derive(Inpt, Debug)]
#[inpt(regex = r"([^ ]+)=(.*?[^ ]+)")]
struct WpaEventKv<'s>(&'s str, &'s str);

#[derive(Debug, Inpt)]
enum WpaEvent<'s> {
    #[inpt(regex = r"<\d>AP-STA-CONNECTED ([A-Za-z\d:]+)")]
    Connected {
        #[inpt(from_str)]
        mac: MacAddr,
        #[inpt(after)]
        kvs: Vec<WpaEventKv<'s>>,
    },
    #[inpt(regex = r"<\d>AP-STA-DISCONNECTED ([A-Za-z\d:]+)")]
    Disconnected {
        #[inpt(from_str)]
        mac: MacAddr,
        #[inpt(after)]
        _kvs: Vec<WpaEventKv<'s>>,
    },
}

#[derive(Debug, Inpt)]
struct WpaStation<'s> {
    #[inpt(from_str, split = "Line")]
    mac: MacAddr,
    kvs: Vec<Line<WpaEventKv<'s>>>,
}

async fn monitor_wpa_events(
    state: WatchState,
    interface: String,
    mut sub: Subscription,
) -> Result<(), Error> {
    while let Ok(msg) = sub.recv().await {
        match inpt(&msg) {
            Ok(WpaEvent::Connected { mac, kvs, .. }) => {
                let mut keyid = None;
                for WpaEventKv(k, v) in kvs {
                    if k == "keyid" {
                        keyid = Some(v.to_string());
                    }
                }
                state.send_modify(|state| {
                    let id = ConnectionId {
                        interface: interface.clone(),
                        mac,
                    };
                    let conn = state.connections.entry(id.clone()).or_default();
                    conn.key_id = keyid;
                    conn.update_profile(&id, &state.config);
                });
            }
            Ok(WpaEvent::Disconnected { mac, .. }) => {
                state.send_modify(|state| {
                    state.connections.remove(&ConnectionId {
                        interface: interface.clone(),
                        mac,
                    });
                });
            }
            Err(_) => (),
        }
    }
    Ok(())
}

async fn monitor_wpa_initial(
    state: WatchState,
    interface: String,
    ctrl: &mut WpaCtrl,
) -> Result<(), Error> {
    let mut sta_str = ctrl.request("STA-FIRST").await?;
    while !sta_str.trim().is_empty() {
        let WpaStation { mac, kvs } = match inpt(&sta_str) {
            Ok(sta) => sta,
            Err(err) => {
                bail!("misformatted station info from wpactrl: {err}");
            }
        };

        let mut keyid = None;
        for Line {
            inner: WpaEventKv(k, v),
        } in kvs
        {
            if k == "keyid" {
                keyid = Some(v.to_string());
            }
        }
        state.send_nomodify(|state| {
            let id = ConnectionId {
                interface: interface.clone(),
                mac,
            };
            let conn = state.connections.entry(id.clone()).or_default();
            conn.key_id = keyid;
            conn.update_profile(&id, &state.config);
        });
        sta_str = ctrl.request(&format!("STA-NEXT {mac}")).await?;
    }
    state.mark_changed();
    Ok(())
}

pub async fn monitor_wpa(state: WatchState, interface: String) -> Result<(), Error> {
    let mut ctrl = WpaCtrl::open(Path::new("/var/run/hostapd").join(&interface))
        .await
        .context("openning hostapd ctrl socket")?;
    match ctrl.request("ATTACH").await?.as_str() {
        "OK" => info!("monitoring wifi interface={interface:?}"),
        err => bail!("ATTACH returned {}", err),
    }
    let monitor = monitor_wpa_events(state.clone(), interface.clone(), ctrl.subscribe());
    let initial = monitor_wpa_initial(state, interface, &mut ctrl);
    try_join!(monitor, initial)?;
    Ok(())
}

#[derive(Debug, Inpt)]
struct AddrWatchEvent<'s> {
    _timestamp: u64,
    #[inpt(split = "Spaced")]
    interface: &'s str,
    #[inpt(split = "Spaced")]
    _vlan_tag: &'s str,
    #[inpt(split = "Spaced", from_str)]
    eth_addr: MacAddr,
    #[inpt(split = "Spaced", from_str)]
    ip_addr: IpAddr,
    #[inpt(split = "Spaced")]
    _pkt_ty: &'s str,
}

pub async fn monitor_addrwatch(state: WatchState, interfaces: Vec<String>) -> Result<(), Error> {
    use tokio::process::Command;

    let addrwatch = Command::new("addrwatch")
        .stdout(Stdio::piped())
        .args(interfaces)
        .spawn()
        .context("spawning addrwatch")?;
    let addrwatch_out = addrwatch
        .stdout
        .ok_or(eyre!("could not connect to hostapd_cli stdout"))?;
    let mut addrwatch_lines = BufReader::new(addrwatch_out).lines();
    while let Some(line) = addrwatch_lines.next_line().await? {
        let Ok(AddrWatchEvent {
            interface,
            eth_addr,
            ip_addr,
            ..
        }) = inpt(&line)
        else {
            error!("could not parse addrwatch line: {:?}", line);
            continue;
        };

        state.send_modify(|state| {
            let conn = state
                .connections
                .entry(ConnectionId {
                    interface: interface.into(),
                    mac: eth_addr,
                })
                .or_default();
            match ip_addr {
                IpAddr::V4(ipv4) => conn.ipv4 = Some(ipv4),
                IpAddr::V6(ipv6) => conn.ipv6 = Some(ipv6),
            }
        });
    }

    Ok(())
}
