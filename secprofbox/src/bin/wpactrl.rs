use secprofbox::{init_logging, wpactrl::WpaCtrl};
use std::io::Write;
use std::{env::args, io::stdout};
use tokio::io::{AsyncBufReadExt as _, BufReader};
use tokio::spawn;
use tracing::{error, info};

#[tokio::main]
pub async fn main() {
    let path = args().nth(1).expect("usage: wpactrl [unix socket path]");
    let _logging = init_logging("wpactrl");
    let mut ctrl = match WpaCtrl::open(&path).await {
        Ok(ctrl) => ctrl,
        Err(err) => {
            eprintln!("could not connect to socket {path:?}: {err}");
            return;
        }
    };
    let mut events = ctrl.subscribe();
    spawn(async move {
        let out = stdout();
        while let Ok(event) = events.recv().await {
            writeln!(out.lock(), "{}", event).unwrap();
        }
    });
    let stdout = stdout();
    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();
    loop {
        let line = match lines.next_line().await {
            Ok(Some(l)) => l,
            Ok(None) => {
                info!("no more lines");
                break;
            }
            Err(err) => {
                eprintln!("could not read stdin: {}", err);
                break;
            }
        };
        match ctrl.request(&line).await {
            Ok(res) => {
                writeln!(stdout.lock(), "{}", res).unwrap();
            }
            Err(err) => eprintln!("could not send command: {}", err),
        }
    }
}
