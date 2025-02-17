use secprofbox::{init_logging, wpactrl::WpaCtrl};
use std::io::Write;
use std::{env::args, io::stdout};
use tokio::io::{AsyncBufReadExt as _, BufReader};
use tokio::spawn;
use tracing::error;

#[tokio::main]
pub async fn main() {
    let path = args().nth(1).expect("usage: wpactrl [unix socket path]");
    init_logging("wpactrl");
    let Ok(mut ctrl) = WpaCtrl::open(&path).await else {
        error!("could not connect to socket {path:?}");
        return;
    };
    let mut events = ctrl.subscribe();
    spawn(async move {
        let out = stdout();
        while let Ok(event) = events.recv().await {
            writeln!(out.lock(), "{}", event).unwrap();
        }
    });
    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();
    let stdout = stdout();
    loop {
        let line = match lines.next_line().await {
            Ok(Some(l)) => l,
            Ok(None) => break,
            Err(err) => {
                error!("could not read stdin: {}", err);
                break;
            }
        };
        match ctrl.request(&line).await {
            Ok(res) => {
                writeln!(stdout.lock(), "{}", res).unwrap();
            }
            Err(err) => error!("could not send command: {}", err),
        }
    }
}
