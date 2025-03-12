use secprofbox::firewall::{maintain_iptables, write_basic_firewall_config};
use secprofbox::monitor::{monitor_addrwatch, monitor_wpa};
use secprofbox::state::{load_config, reload_config_sighup, State};
use secprofbox::{init_logging, state::WatchState};
use tokio::task::JoinSet;
use tracing::error;

#[tokio::main]
pub async fn main() {
    let _logging = init_logging("secprofdebug");
    let mut tasks = JoinSet::new();
    let mut state = State::default();
    match load_config() {
        Ok(cfg) => state.config = cfg,
        Err(err) => {
            println!("{:?}", err);
            error!("{:?}", err);
            return;
        }
    }
    if let Err(err) = write_basic_firewall_config(&state.config).await {
        error!("could not ensure /etc/config/firewall is up to date: {err}");
    }

    let state = WatchState::new(state);
    tasks.spawn(reload_config_sighup(state.clone()));
    tasks.spawn(maintain_iptables(state.clone()));
    tasks.spawn(monitor_wpa(state.clone(), "phy0-ap0".into()));
    tasks.spawn(monitor_addrwatch(state.clone(), vec!["phy0-ap0".into()]));

    loop {
        match tasks.join_next().await {
            Some(Err(err)) => {
                println!("shutting down secprof because of panic {:?}", err);
                error!("shutting down secprof because of panic {:?}", err);
                return;
            }
            Some(Ok(Err(err))) => {
                println!("shutting down secprof because of error {:?}", err);
                error!("shutting down secprof because of error {:?}", err);
                return;
            }
            Some(Ok(_)) => (),
            None => return,
        }
    }
}
