use color_eyre::eyre::Error;
use secprofbox::firewall::produce_rule_changes;
use secprofbox::monitor::{monitor_addrwatch, monitor_wpa};
use secprofbox::state::{KeyId, LanAccess, SecProfile, State};
use secprofbox::{init_logging, state::WatchState};
use tokio::task::JoinSet;
use tracing::error;

pub async fn log_state(mut state: WatchState) -> Result<(), Error> {
    state
        .wait_for(|state| {
            println!("STATE={:#?}", state);
            false
        })
        .await;
    Ok(())
}

pub async fn log_firewall(state: WatchState) -> Result<(), Error> {
    produce_rule_changes(state, |change| async move {
        println!("{:?}", change.iptables(),);
        Ok(())
    })
    .await?;
    Ok(())
}

#[tokio::main]
pub async fn main() {
    let _logging = init_logging("secprofdebug");
    let mut tasks = JoinSet::new();
    let mut state = State::default();
    state.config.profiles.insert(
        "default".into(),
        SecProfile {
            lan: LanAccess::AllDevices,
            wan: true,
        },
    );
    state.config.profiles.insert(
        "guest".into(),
        SecProfile {
            lan: LanAccess::NoDevices,
            wan: true,
        },
    );
    state.config.profiles.insert(
        "foo".into(),
        SecProfile {
            lan: LanAccess::OtherProfile(vec!["foo".into()]),
            wan: true,
        },
    );
    state.config.keyids.insert(
        "guest".into(),
        KeyId {
            profile: "guest".into(),
            password: "".into(),
        },
    );
    state.config.keyids.insert(
        "foo".into(),
        KeyId {
            profile: "foo".into(),
            password: "".into(),
        },
    );
    state.config.keyids.insert(
        "default".into(),
        KeyId {
            profile: "default".into(),
            password: "".into(),
        },
    );
    let state = WatchState::new(state);

    //tasks.spawn(log_state(state.clone()));
    tasks.spawn(log_firewall(state.clone()));
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
