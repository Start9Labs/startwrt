use color_eyre::eyre::Error;
use secprofbox::firewall::generate_allows;
use secprofbox::monitor::{monitor_addrwatch, monitor_wpa};
use secprofbox::state::{LanAccess, SecProfile, State};
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

pub async fn log_firewall(mut state: WatchState) -> Result<(), Error> {
    state
        .wait_for(|state| {
            let mut allows = Vec::new();
            generate_allows(state, &mut allows);
            allows.sort();
            println!("ALLOW={:#?}", allows);
            false
        })
        .await;
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
    state
        .config
        .keyid_to_profile
        .insert("guest".into(), "guest".into());
    state
        .config
        .keyid_to_profile
        .insert("foo".into(), "foo".into());
    state
        .config
        .keyid_to_profile
        .insert("default".into(), "default".into());
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
