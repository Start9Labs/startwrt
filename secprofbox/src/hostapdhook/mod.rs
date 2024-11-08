use color_eyre::eyre::{bail, eyre, Error};
use rust_uci::Uci;
use std::collections::VecDeque;
use std::ffi::OsString;
use std::io::{stdin, Read};
use tracing::{debug, error};

#[derive(Debug)]
struct Connected<'s> {
    interface: &'s str,
    mac: &'s str,
    keyid: Option<&'s str>,
}

fn parse_hostapd_connected(msg: &str) -> Result<Option<Connected>, Error> {
    let (interface, msg) = msg
        .split_once(' ')
        .ok_or(eyre!("eof after interface in {msg:?}"))?;
    if !msg.starts_with("AP-STA-CONNECTED") {
        return Ok(None);
    }
    let (_event, msg) = msg
        .split_once(' ')
        .ok_or(eyre!("eof after event in {msg:?}"))?;
    let mut keyid = None;
    let mac = match msg.split_once(' ') {
        Some((mac, msg)) => {
            for kv in msg.split(' ') {
                let (k, v) = kv
                    .split_once('=')
                    .ok_or(eyre!("{kv:?} not of the form k=v"))?;
                if k == "keyid" {
                    keyid = Some(v);
                }
            }
            mac
        }
        None => msg,
    };
    Ok(Some(Connected {
        interface,
        mac,
        keyid,
    }))
}

fn handle_connected(
    Connected {
        interface,
        mac,
        keyid,
    }: Connected,
) -> Result<(), Error> {
}

pub fn main(_args: VecDeque<OsString>) {
    let mut line = String::new();
    if let Err(err) = stdin().lock().read_to_string(&mut line) {
        error!("could not read stdin: {err}");
        return;
    }

    let msg = match parse_hostapd_connected(&line) {
        Ok(Some(msg)) => msg,
        Ok(None) => return,
        Err(err) => {
            error!("parsing hostapd failed: {err}");
            return;
        }
    };

    debug!("observed {msg:?}");

    if let Err(err) = handle_connected(msg) {
        error!("update failed: {err}");
        return;
    }
}
