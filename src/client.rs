mod methods;
mod receiver;

use crate::error::{Error, ProtocolViolation, Result};
use crate::msg::{inbound, outbound};
use receiver::Receiver;

use std::{ffi::OsStr, process::Stdio};

use tokio::{
    io::AsyncWriteExt,
    process::{Child, ChildStdin, Command},
    sync::{broadcast, mpsc, oneshot},
};

use lsp_types::notification::Notification as NotificationTrait;
use lsp_types::request::Request as RequestTrait;

#[derive(Debug)]
pub struct Client {
    proc: Child,
    stdin: ChildStdin,
    req_id: u32,
    reqs_tx: mpsc::Sender<(u32, inbound::RpcSender)>,
    notifs_rx: broadcast::Receiver<inbound::Notification>,
}

impl Client {
    pub fn new<S: AsRef<OsStr>>(program: S) -> std::io::Result<Self> {
        let mut proc = Command::new(program)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let stdout = proc.stdout.take().unwrap();
        let (reqs_tx, reqs_rx) = mpsc::channel(100);
        let (notifs_tx, notifs_rx) = broadcast::channel(1000);
        let receiver = Receiver::new(stdout, reqs_rx, notifs_tx);
        tokio::spawn(receiver.run());

        Ok(Self {
            stdin: proc.stdin.take().unwrap(),
            proc,
            req_id: 1,
            reqs_tx,
            notifs_rx,
        })
    }

    fn req_id(&mut self) -> u32 {
        let id = self.req_id;
        self.req_id += 1;
        id
    }

    fn build_msg(content: impl outbound::Message) -> serde_json::Result<String> {
        let content_part = serde_json::to_string(&content)?;
        let lines = [
            format!("Content-Length: {}", content_part.len()),
            String::new(),
            content_part,
        ];
        Ok(lines.join("\r\n"))
    }

    async fn send_msg(&mut self, msg: &[u8]) -> std::io::Result<()> {
        self.stdin.write_all(msg).await
    }

    pub async fn request<R: RequestTrait>(&mut self, params: R::Params) -> Result<R::Result>
    where
        R: RequestTrait,
        R::Result: 'static,
    {
        let id = self.req_id();
        let content = outbound::Request::<R> { id, params };

        let (tx, rx) = oneshot::channel();
        self.reqs_tx.send((id, tx)).await.unwrap();

        let msg = Self::build_msg(content).unwrap();
        self.send_msg(msg.as_bytes())
            .await
            .map_err(Error::SendMsg)?;

        let response: inbound::GenericResponse = rx.await.expect("oneshot closed");

        assert_eq!(response.id, id);
        match (response.result, response.error) {
            (Some(r), None) => Ok(serde_json::from_value(r).expect("bad data")),
            (None, Some(e)) => Err(Error::Lsp(e)),
            (Some(_), Some(_)) => Err(ProtocolViolation::BothResultAndResponse.into()),
            (None, None) => {
                // Hang on.
                // If R::Result gets serialized as a unit,
                // then we could've deserialized it as None instead of Some(()).
                // Don't be so hasty to return an error.
                if std::any::TypeId::of::<R::Result>() == std::any::TypeId::of::<()>() {
                    Ok(serde_json::from_value(().into()).unwrap())
                } else {
                    Err(ProtocolViolation::NeitherResultNorResponse.into())
                }
            }
        }
    }

    pub async fn notify<N: NotificationTrait>(&mut self, params: N::Params) -> Result<()> {
        let content = outbound::Notification::<N> { params };
        let msg = Self::build_msg(content).unwrap();
        self.send_msg(msg.as_bytes())
            .await
            .map_err(Error::SendMsg)?;
        Ok(())
    }
}
