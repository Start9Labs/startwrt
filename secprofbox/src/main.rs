use std::collections::VecDeque;
use std::ffi::OsString;
use std::path::Path;
use std::process::ExitCode;
use tracing::subscriber::DefaultGuard;

#[cfg(feature = "secprof-map")]
mod map;
#[cfg(feature = "secprof-watchwifi")]
mod watchwifi;

pub mod dumbuci;

fn select_executable(name: &str) -> Option<fn(VecDeque<OsString>) -> ExitCode> {
    match name {
        #[cfg(feature = "secprof-watchwifi")]
        "secprof-watchwifi" => Some(watchwifi::main),
        #[cfg(feature = "secprof-map")]
        "secprof-map" => Some(map::main),
        _ => None,
    }
}

fn init_logging(name: &str) -> DefaultGuard {
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

pub fn main() -> ExitCode {
    let mut args = std::env::args_os().collect::<VecDeque<_>>();
    for _ in 0..2 {
        let Some(s) = args.pop_front() else { break };
        let Some(name) = Path::new(&*s).file_name().and_then(|s| s.to_str()) else {
            break;
        };
        let Some(x) = select_executable(name) else {
            continue;
        };
        let _logging_guard = init_logging(name);
        args.push_front(s);
        return x(args);
    }
    let args = std::env::args().collect::<VecDeque<_>>();
    eprintln!(
        "unknown executable: {}",
        args.get(1)
            .or_else(|| args.get(0))
            .map(|s| s.as_str())
            .unwrap_or("N/A")
    );
    ExitCode::from(1)
}
