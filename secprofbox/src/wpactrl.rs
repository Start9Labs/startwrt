use async_cell::sync::AsyncCell;
use color_eyre::eyre::Error;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio::task::JoinHandle;
use tracing::{error, info};

const BUF_SIZE: usize = 10_240;

struct WpaCtrlInner {
    event_sender: Sender<String>,
    response: AsyncCell<String>,
    socket: tokio::net::UnixDatagram,
}

impl WpaCtrlInner {
    async fn process_socket(&self) {
        let mut buf = [0u8; BUF_SIZE];

        loop {
            let len = match self.socket.recv(&mut buf).await {
                Ok(len) => len,
                Err(e) => {
                    error!("Failed to read from socket: {}", e);
                    break;
                }
            };

            let msg = match std::str::from_utf8(&buf[..len]) {
                Ok(msg) => msg.trim().to_string(),
                Err(err) => {
                    error!("Socket provided non-utf8 bytes: {}", err);
                    continue;
                }
            };

            if msg.starts_with('<') {
                info!("Event: {}", msg);
                if self.event_sender.send(msg).is_err() {
                    // Drop the event
                    continue;
                }
            } else {
                info!("Response: {}", msg);
                // Signal completion of the request
                self.response.set(msg);
            }
        }
    }
}

pub struct WpaCtrl {
    inner: Arc<WpaCtrlInner>,
    task: JoinHandle<()>,
    bind_filepath: PathBuf,
    ctrl_filepath: PathBuf,
}

static COUNTER: AtomicU64 = AtomicU64::new(1);

impl WpaCtrl {
    pub async fn open<P: AsRef<Path>>(ctrl_filepath: P) -> Result<Self, Error> {
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
        let ctrl_filepath = ctrl_filepath.as_ref().to_path_buf();
        let bind_filename = format!("wpa_ctrl_{}-{}", std::process::id(), counter);
        let bind_filepath = Path::new("/tmp").join(bind_filename);

        let socket = tokio::net::UnixDatagram::bind(&bind_filepath)?;
        socket.connect(&ctrl_filepath)?;

        let (event_sender, _) = broadcast::channel(8);
        let response = AsyncCell::new();

        let inner = Arc::new(WpaCtrlInner {
            event_sender,
            response,
            socket,
        });

        let task_inner = inner.clone();
        let task = tokio::spawn(async move { task_inner.process_socket().await });

        Ok(Self {
            inner,
            task,
            bind_filepath,
            ctrl_filepath,
        })
    }

    pub fn paths(&self) -> (&Path, &Path) {
        (&self.bind_filepath, &self.ctrl_filepath)
    }

    pub fn subscribe(&self) -> Receiver<String> {
        self.inner.event_sender.subscribe()
    }

    pub async fn request(&mut self, command: &str) -> Result<String, Error> {
        self.inner.socket.send(command.as_bytes()).await?;
        info!("Sent command: {}", command);
        Ok(self.inner.response.take().await)
    }
}

impl Drop for WpaCtrl {
    fn drop(&mut self) {
        self.task.abort();
        let _ = self.inner.socket.shutdown(std::net::Shutdown::Both);
        let _ = std::fs::remove_file(&self.bind_filepath);
    }
}
