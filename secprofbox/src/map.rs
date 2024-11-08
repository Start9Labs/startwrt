use std::{collections::VecDeque, ffi::OsString, net::Ipv4Addr, process::ExitCode};

pub struct SecurityProfileId(pub String);

pub struct SecurityProfile {
    id: SecurityProfileId,
    allowed_profile_src: Vec<SecurityProfileId>,
    allow_wan: bool,
}

pub struct Zone(pub String);

pub enum Target {
    Accept,
    Reject,
    Drop,
}

#[derive(Clone)]
pub struct FirewallFilter {
    zone: Zone,
    ip: Option<Ipv4Addr>,
}

pub struct FirewallRule {
    src: FirewallFilter,
    dest: FirewallFilter,
    target: Target,
}
impl std::fmt::Display for FirewallRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let FirewallRule {
            src:
                FirewallFilter {
                    zone: src_zone,
                    ip: src_ip,
                },
            dest:
                FirewallFilter {
                    zone: dest_zone,
                    ip: dest_ip,
                },
            target,
        } = self;
        writeln!(f, "config rule")?;
        writeln!(f, "\toption src {}", src_zone.0)?;
        if let Some(ip) = src_ip {
            writeln!(f, "\toption src_ip {}", ip)?;
        }
        writeln!(f, "\toption dest {}", dest_zone.0)?;
        if let Some(ip) = dest_ip {
            writeln!(f, "\toption dest_ip {}", ip)?;
        }
        match target {
            Target::Accept => writeln!(f, "\toption target accept")?,
            Target::Reject => writeln!(f, "\toption target reject")?,
            Target::Drop => writeln!(f, "\toption target drop")?,
        }

        Ok(())
    }
}

pub enum ConnectionMethod {
    Ethernet {
        interface: Zone,
    },
    VPN {
        interface: Zone,
        ipv4: Ipv4Addr, // TODO: ip6only?
                        // ipv6: Ipv6Addr,
    },
    WiFi {
        keyid: String,
    },
}

pub trait SecurityProfileResolver {
    fn resolve_filters(&self, method: &ConnectionMethod) -> Vec<FirewallFilter>;
    fn resolve_profile(&self, method: &ConnectionMethod) -> SecurityProfile;
    fn resolve_connection_methods(&self, profile: &SecurityProfileId) -> Vec<ConnectionMethod>;
    fn resolve_wan_zones(&self) -> Vec<Zone>;
}

pub fn make_rules(
    resolver: &impl SecurityProfileResolver,
    known_connection_methods: &[ConnectionMethod],
) -> Vec<FirewallRule> {
    let mut res = Vec::new();

    for method in known_connection_methods {
        let profile = resolver.resolve_profile(method);
        for src_profile in &profile.allowed_profile_src {
            for src_method in resolver.resolve_connection_methods(src_profile) {
                for src_filter in resolver.resolve_filters(&src_method) {
                    for dest_filter in resolver.resolve_filters(method) {
                        res.push(FirewallRule {
                            src: src_filter.clone(),
                            dest: dest_filter,
                            target: Target::Accept,
                        });
                    }
                }
            }
        }
    }

    res
}

pub fn main(args: VecDeque<OsString>) -> ExitCode {
    todo!()
}
