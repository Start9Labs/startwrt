use color_eyre::eyre::{bail, eyre, Context, Error};
use inpt::{inpt, Inpt};
use std::net::IpAddr;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{error, info};

use crate::state::{WatchState, WifiConnectionId};
use crate::wpactrl::WpaCtrl;

#[derive(Inpt, Debug)]
#[inpt(regex = r"([^ ]+)=([^ ]+)")]
struct WpaEventKv<'s>(&'s str, &'s str);

#[derive(Debug, Inpt)]
enum WpaEvent<'s> {
    #[inpt(regex = r"<(\d)>AP-STA-CONNECTED ([A-Za-z\d:]+)")]
    Connected {
        _level: u8,
        mac: &'s str,
        #[inpt(after)]
        kvs: Vec<WpaEventKv<'s>>,
    },
    #[inpt(regex = r"<(\d)>AP-STA-DISCONNECTED ([A-Za-z\d:]+)")]
    Disconnected {
        _level: u8,
        mac: &'s str,
        #[inpt(after)]
        _kvs: Vec<WpaEventKv<'s>>,
    },
}

pub async fn monitor_wpa(state: WatchState, interface: String) -> Result<(), Error> {
    let mut ctrl = WpaCtrl::open(Path::new("/var/run/hostapd").join(&interface))
        .await
        .context("openning hostapd ctrl socket")?;
    match ctrl.request("ATTACH").await?.as_str() {
        "OK" => info!("monitoring wifi interface={interface:?}"),
        err => bail!("ATTACH returned {}", err),
    }
    let mut sub = ctrl.subscribe();
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
                    state
                        .wifi_connections
                        .entry(WifiConnectionId {
                            interface: interface.clone(),
                            mac: mac.into(),
                        })
                        .or_default()
                        .key_id = keyid;
                });
            }
            Ok(WpaEvent::Disconnected { mac, .. }) => {
                state.send_modify(|state| {
                    state.wifi_connections.remove(&WifiConnectionId {
                        interface: interface.clone(),
                        mac: mac.into(),
                    });
                });
            }
            Err(_) => (),
        }
    }
    // NOTE: we can still send requests to ctrl, if we need to force a disconnect or something
    Ok(())
}

#[derive(Debug, Inpt)]
struct AddrWatchEvent<'s> {
    _timestamp: u64,
    #[inpt(split = "Spaced")]
    interface: &'s str,
    #[inpt(split = "Spaced")]
    _vlan_tag: &'s str,
    #[inpt(split = "Spaced")]
    eth_addr: &'s str,
    #[inpt(split = "Spaced", from_str)]
    ip_addr: IpAddr,
    #[inpt(split = "Spaced")]
    _pkt_ty: &'s str,
}

pub async fn monitor_addrwatch(state: WatchState) -> Result<(), Error> {
    use tokio::process::Command;

    let addrwatch = Command::new("addrwatch")
        .stdout(Stdio::piped())
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
            state
                .wifi_connections
                .entry(WifiConnectionId {
                    interface: interface.into(),
                    mac: eth_addr.into(),
                })
                .or_default()
                .ip = Some(ip_addr);
        });
    }

    Ok(())
}
