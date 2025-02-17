use std::collections::VecDeque;
use std::ffi::OsString;
use std::process::ExitCode;
use tracing::subscriber::DefaultGuard;

#[cfg(feature = "secprof-map")]
mod map;
#[cfg(feature = "secprof-watchwifi")]
mod watchwifi;

pub mod state;
pub mod wpactrl;

pub fn select_executable(name: &str) -> Option<fn(VecDeque<OsString>) -> ExitCode> {
    match name {
        #[cfg(feature = "secprof-watchwifi")]
        "secprof-watchwifi" => Some(watchwifi::main),
        #[cfg(feature = "secprof-map")]
        "secprof-map" => Some(map::main),
        _ => None,
    }
}

pub fn init_logging(name: &str) -> DefaultGuard {
    use tracing_rfc_5424::{
        rfc3164::Rfc3164, tracing::TrivialTracingFormatter, transport::UnixSocket,
    };
    use tracing_subscriber::Registry;
    use tracing_subscriber::{
        layer::SubscriberExt, // Needed to get `with()`
    };

    // Setup the subsriber...
    let subscriber = Registry::default().with(
        tracing_rfc_5424::layer::Layer::<
            tracing_subscriber::Registry,
            Rfc3164,
            TrivialTracingFormatter,
            UnixSocket,
        >::try_default()
        .unwrap(),
    );
    // and install it.
    tracing::subscriber::set_default(subscriber)
}
