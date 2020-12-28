use crate::error::{Error, Result};
use crate::msg::{IncomingNotification, Message, RpcSender};

use std::collections::HashMap;

use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    process::ChildStdout,
    sync::{broadcast, mpsc},
};

use either::Either;
use futures::FutureExt;
use serde_json::Value;

pub struct Receiver {
    stdout: BufReader<ChildStdout>,
    listeners: mpsc::Receiver<(u32, RpcSender)>,
    notifs: broadcast::Sender<IncomingNotification>,
    channels: HashMap<u32, RpcSender>,
}

impl Receiver {
    async fn recv_bytes(&mut self) -> std::io::Result<Vec<u8>> {
        let mut content_len = None;
        let mut line = String::new();
        loop {
            self.stdout.read_line(&mut line).await?;
            if line == "\r\n" {
                break;
            } else if let Some(len) = line.strip_prefix("Content-Length: ") {
                let n = len.trim_end().parse::<usize>().expect("bad num");
                content_len = Some(n);
            }
            line.clear();
        }
        let mut buf = vec![0; content_len.expect("no len")];
        self.stdout.read_exact(&mut buf).await?;
        Ok(buf)
    }

    async fn recv_msg(&mut self) -> Result<Message> {
        let bytes = self.recv_bytes().await.map_err(Error::RecvMsg)?;
        let decoded: Value = serde_json::from_slice(&bytes).expect("bad json");
        if decoded.get("id").is_some() {
            let response = serde_json::from_value(decoded).expect("bad data");
            Ok(Either::Left(response))
        } else {
            let notification = serde_json::from_value(decoded).expect("bad data");
            Ok(Either::Right(notification))
        }
    }

    fn update_listeners(&mut self) {
        while let Some(res) = self.listeners.recv().now_or_never() {
            let (id, listener) = res.expect("rpc listeners channel closed");
            let old_chan = self.channels.insert(id, listener);
            assert!(old_chan.is_none(), "Request ID conflict for ID {}", id);
        }
    }

    fn handle_msg(&mut self, msg: Message) {
        match msg {
            Either::Left(response) => {
                self.update_listeners();
                if let Some(chan) = self.channels.remove(&response.id) {
                    chan.send(response).unwrap_or(());
                } else {
                    eprintln!("Received response for nonexistent request: {:?}", response);
                }
            }
            Either::Right(notif) => {
                self.notifs.send(notif).unwrap_or(0);
            }
        }
    }

    pub fn new(
        stdout: ChildStdout,
        listeners: mpsc::Receiver<(u32, RpcSender)>,
        notifs: broadcast::Sender<IncomingNotification>,
    ) -> Self {
        Self {
            stdout: BufReader::new(stdout),
            listeners,
            notifs,
            channels: HashMap::new(),
        }
    }

    pub async fn run(mut self) {
        loop {
            let msg = self.recv_msg().await.expect("failed to get msg");
            self.handle_msg(msg);
        }
    }
}
