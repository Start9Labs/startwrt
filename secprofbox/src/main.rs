use secprofbox::{init_logging, select_executable};

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
