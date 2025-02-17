use color_eyre::eyre::{bail, eyre, Context, Error};
use inpt::{inpt, Inpt};
use std::collections::VecDeque;
use std::ffi::OsString;
use std::process::{ExitCode, Stdio};
use tokio::io::AsyncBufReadExt;
use tokio::select;
use tracing::{debug, error, warn};

#[derive(Inpt, Debug)]
#[inpt(regex = r"([^ ]+)=([^ ]+)")]
struct ConnectedKv<'s>(&'s str, &'s str);

#[derive(Debug, Inpt)]
#[inpt(regex = r"([^ ]+) AP-STA-CONNECTED ([A-Za-z\d:]+)")]
struct Connected<'s> {
    interface: &'s str,
    mac: &'s str,
    #[inpt(after)]
    kvs: Vec<ConnectedKv<'s>>,
}

#[derive(Debug, Inpt)]
struct Pairing<'s> {
    timestamp: u64,
    #[inpt(split = "Spaced")]
    interface: &'s str,
    #[inpt(split = "Spaced")]
    vlan_tag: &'s str,
    #[inpt(split = "Spaced")]
    eth_addr: &'s str,
    #[inpt(split = "Spaced")]
    ip_addr: &'s str,
    #[inpt(split = "Spaced")]
    pkt_ty: &'s str,
}

/*
// this state machine came out a bit more crazy than I expected
struct UpdateWifiConnection {
    out: WriteUci,
    in_wifi_connection: bool,
    clear_keyid: bool,
    update_interface: String,
    update_mac: String,
    update_keyid: Option<String>,
    update_ip: Option<String>,
    at_interface: Option<String>,
    at_mac: Option<String>,
    at_keyid: Option<String>,
    at_ip: Option<String>,
}

impl AsRef<WriteUci> for UpdateWifiConnection {
    fn as_ref(&self) -> &WriteUci {
        &self.out
    }
}

impl VisitUci for UpdateWifiConnection {
    fn enter_section(&mut self, ty: &str, name: Option<&str>) -> Result<(), Error> {
        self.in_wifi_connection = ty == "wifi_connection";
        self.at_interface = None;
        self.at_mac = None;
        self.at_keyid = None;
        self.at_ip = None;
        if !self.in_wifi_connection {
            self.out.enter_section(ty, name);
        }
        Ok(())
    }

    fn exit_section(&mut self) -> Result<(), Error> {
        if !self.in_wifi_connection {
            return self.out.exit_section();
        }

        if self.at_mac.as_deref() == Some(self.update_mac.as_str()) {
            self.update_ip = self.update_ip.take().or(self.at_ip.take());
            self.update_keyid = self.at_keyid.take().or(self.at_keyid.take());
        } else if self.update_ip.is_none() || self.at_ip != self.update_ip {
            self.out.enter_section("wifi_connection", None)?;
            if let Some(interface) = &self.at_interface {
                self.out.option("interface", interface)?;
            }
            if let Some(mac) = &self.at_mac {
                self.out.option("mac_addr", mac)?;
            }
            if let Some(ip) = &self.at_ip {
                self.out.option("ip_addr", ip)?;
            }
            if let Some(keyid) = &self.at_keyid {
                self.out.option("keyid", keyid)?;
            }
            return self.out.exit_section();
        }
        Ok(())
    }

    fn option(&mut self, option: &str, value: &str) -> Result<(), Error> {
        if !self.in_wifi_connection {
            return self.out.option(option, value);
        }

        match option {
            "interface" => {
                self.at_interface = Some(value.to_owned());
            }
            "mac_addr" => {
                self.at_mac = Some(value.to_owned());
            }
            "ip_addr" => {
                self.at_ip = Some(value.to_owned());
            }
            "keyid" => {
                self.at_keyid = Some(value.to_owned());
            }
            _ => (),
        }
        Ok(())
    }

    fn list(&mut self, list: &str, item: &str) -> Result<(), Error> {
        if !self.in_wifi_connection {
            return self.out.list(list, item);
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<(), Error> {
        self.out.enter_section("wifi_connection", None)?;
        self.out.option("interface", &self.update_interface)?;
        self.out.option("mac_addr", &self.update_mac)?;
        if let Some(ip) = &self.update_ip {
            self.out.option("ip_addr", ip)?;
        }
        if !self.clear_keyid {
            if let Some(keyid) = &self.update_keyid {
                self.out.option("keyid", keyid)?;
            }
        }
        self.out.exit_section();
        Ok(())
    }
}
*/

struct Actor {
    // TODO: do we actually need any in-memory state here?
}

impl Actor {
    fn handle_connected(
        &mut self,
        Connected {
            interface,
            mac,
            kvs,
        }: Connected,
    ) -> Result<(), Error> {
        let keyid = kvs.iter().find_map(|&ConnectedKv(k, v)| {
            if k == "keyid" {
                Some(v.to_owned())
            } else {
                None
            }
        });
        /*rewrite_config("/etc/config/secprofstate", |out| UpdateWifiConnection {
            out,
            clear_keyid: keyid.is_none(),
            in_wifi_connection: false,
            update_interface: interface.to_owned(),
            update_mac: mac.to_owned(),
            update_keyid: keyid,
            update_ip: None,
            at_interface: None,
            at_mac: None,
            at_keyid: None,
            at_ip: None,
        });*/
        Ok(())
    }

    fn handle_pairing(
        &mut self,
        Pairing {
            interface,
            eth_addr,
            ip_addr,
            ..
        }: Pairing,
    ) -> Result<(), Error> {
        /*rewrite_config("/etc/config/secprofstate", |out| UpdateWifiConnection {
            out,
            clear_keyid: false,
            in_wifi_connection: false,
            update_interface: interface.to_owned(),
            update_mac: eth_addr.to_owned(),
            update_keyid: None,
            update_ip: Some(ip_addr.to_owned()),
            at_interface: None,
            at_mac: None,
            at_keyid: None,
            at_ip: None,
        });*/
        Ok(())
    }
}

pub async fn start(actor: &mut Actor) -> Result<(), Error> {
    use tokio::io::BufReader;
    use tokio::process::Command;

    // TODO: monitor hostapd socket ourselves
    let addrwatch = Command::new("hostapd_cli").stdout(Stdio::piped()).spawn()?;
    let hostapd_out = addrwatch
        .stdout
        .ok_or(eyre!("could not connect to hostapd_cli stdout"))?;
    let mut hostapd_lines = BufReader::new(hostapd_out).lines();

    let addrwatch = Command::new("addrwatch").stdout(Stdio::piped()).spawn()?;
    let addrwatch_out = addrwatch
        .stdout
        .ok_or(eyre!("could not connect to hostapd_cli stdout"))?;
    let mut addrwatch_lines = BufReader::new(addrwatch_out).lines();

    loop {
        select! {
            line = hostapd_lines.next_line() => {
                let Some(line) = line.context("reading from hostapd")? else { bail!("hostapd shutdown unexpectedly") };
                let msg = match inpt(&line) {
                    Ok(msg) => msg,
                    Err(err) => {
                        debug!("hostapd log {line:?} did not match AP-STA-CONNECTED: {err}");
                        continue;
                    }
                };
                debug!("hostapd observed: {msg:?}");
                if let Err(err) = actor.handle_connected(msg) {
                    error!("could not handle connection event {line:?}: {err:?}");
                }
            },
            line = addrwatch_lines.next_line() => {
                let Some(line) = line.context("reading from addrwatch")? else { bail!("addrwatch shutdown unexpectedly") };
                let msg = match inpt(&line) {
                    Ok(msg) => msg,
                    Err(err) => {
                        warn!("could not parse addrwatch line {line:?}: {err}");
                        continue;
                    }
                };
                debug!("addrwatch observed: {msg:?}");
                if let Err(err) = actor.handle_pairing(msg) {
                    error!("could not handle pairing event {line:?}: {err:?}");
                }
            },
        }
    }
}

pub fn main(_args: VecDeque<OsString>) -> ExitCode {
    use tokio::runtime::Builder;
    let Ok(rt) = Builder::new_current_thread().build() else {
        error!("could not start tokio runtime");
        return ExitCode::from(1);
    };
    rt.block_on(async {
        let mut actor = Actor {};
        if let Err(err) = start(&mut actor).await {
            error!("error while watching wifi connections: {err:?}");
            return ExitCode::from(1);
        }
        ExitCode::from(0)
    })
}
