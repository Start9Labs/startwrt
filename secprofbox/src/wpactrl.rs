use async_cell::sync::AsyncCell;
use color_eyre::eyre::Error;
use std::ffi::CString;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::UnixStream;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio::task::JoinHandle;
use tracing::{error, info};

const BUF_SIZE: usize = 10_240;

struct WpaCtrlInner {
    event_sender: Sender<String>,
    response: AsyncCell<String>,
}

impl WpaCtrlInner {
    async fn process_socket(&self, mut socket_reader: OwnedReadHalf) {
        let mut buf = [0u8; BUF_SIZE];

        loop {
            let len = match socket_reader.readable().await {
                Ok(_) => match socket_reader.read(&mut buf).await {
                    Ok(len) => len,
                    Err(e) => {
                        error!("Failed to read from socket: {}", e);
                        break;
                    }
                },
                Err(e) => {
                    error!("Socket became unreadable: {}", e);
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
                if let Err(_) = self.event_sender.send(msg) {
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
    socket_writer: OwnedWriteHalf,
}

impl WpaCtrl {
    pub async fn open<P: AsRef<Path>>(ctrl_path: P) -> Result<Self, Error> {
        let ctrl_path = ctrl_path.as_ref().to_path_buf();
        let socket = UnixStream::connect(ctrl_path).await?;
        let (socket_reader, socket_writer) = socket.into_split();
        let (event_sender, _) = broadcast::channel(8);
        let response = AsyncCell::new();

        let inner = Arc::new(WpaCtrlInner {
            event_sender,
            response,
        });

        let task_inner = inner.clone();
        let task = tokio::spawn(async move { task_inner.process_socket(socket_reader).await });

        Ok(Self {
            inner,
            task,
            socket_writer,
        })
    }

    pub fn subscribe(&self) -> Receiver<String> {
        self.inner.event_sender.subscribe()
    }

    pub async fn request(&mut self, command: &str) -> Result<String, Error> {
        info!("Sending command: {}", command);
        self.socket_writer.writable().await?;
        self.socket_writer
            .write_all(
                CString::new(command)
                    .expect("request not a valid cstring")
                    .as_bytes_with_nul(),
            )
            .await?;
        Ok(self.inner.response.take().await)
    }
}

impl Drop for WpaCtrl {
    fn drop(&mut self) {
        self.task.abort();
    }
}
