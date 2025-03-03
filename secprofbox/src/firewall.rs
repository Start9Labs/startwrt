use crate::state::{Connection, ConnectionId, LanAccess, SecProfile, State};
use std::net::IpAddr;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Zone {
    Lan,
    Wan,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct AllowRule {
    pub src_zone: Zone,
    pub src_ip: Option<IpAddr>,
    pub src_mac: Option<String>,
    pub dest_zone: Zone,
    pub dest_ip: Option<IpAddr>,
}

pub fn generate_profile2profile_allows(
    state: &State,
    src_ip: IpAddr,
    src_mac: &str,
    dst_profile: &str,
    allows: &mut Vec<AllowRule>,
) {
    for Connection {
        profile,
        ipv4,
        ipv6,
        ..
    } in state.connections.values()
    {
        if profile.as_deref() != Some(dst_profile) {
            continue;
        }

        let ips = [ipv4.map(IpAddr::from), ipv6.map(IpAddr::from)]
            .into_iter()
            .flatten();

        for ip in ips {
            allows.push(AllowRule {
                src_zone: Zone::Lan,
                src_ip: Some(src_ip),
                src_mac: Some(src_mac.into()),
                dest_zone: Zone::Lan,
                dest_ip: Some(ip),
            })
        }
    }
}

pub fn generate_allows(state: &State, allows: &mut Vec<AllowRule>) {
    for (
        ConnectionId { mac, .. },
        Connection {
            profile,
            ipv4,
            ipv6,
            ..
        },
    ) in state.connections.iter()
    {
        // TODO: use a zone for interface profiles, instead of doing the ip<->ip thing

        let Some(profile) = profile else { continue };
        let ips = [ipv4.map(IpAddr::from), ipv6.map(IpAddr::from)]
            .into_iter()
            .flatten();

        let Some(SecProfile { lan, wan }) = state.config.profiles.get(profile) else {
            continue;
        };

        for ip in ips {
            match wan {
                false => (),
                true => allows.push(AllowRule {
                    src_zone: Zone::Lan,
                    src_ip: Some(ip),
                    src_mac: Some(mac.clone()),
                    dest_zone: Zone::Wan,
                    dest_ip: None,
                }),
            }

            match lan {
                LanAccess::AllDevices => allows.push(AllowRule {
                    src_zone: Zone::Lan,
                    src_ip: Some(ip),
                    src_mac: Some(mac.clone()),
                    dest_zone: Zone::Lan,
                    dest_ip: None,
                }),
                LanAccess::NoDevices => (),
                LanAccess::OtherProfile(dst_profiles) => {
                    for dst_profile in dst_profiles {
                        generate_profile2profile_allows(state, ip, mac, dst_profile, allows);
                    }
                }
            }
        }
    }
}
