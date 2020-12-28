use crate::error::{Error, ProtocolViolation, Result};
use crate::msg::{Notification, OutboundMessage, Request, Response};

use std::{ffi::OsStr, process::Stdio};

use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
};

use lsp_types::notification::Notification as NotificationTrait;
use lsp_types::request::Request as RequestTrait;
use lsp_types::{
    notification::Initialized, request::Initialize, ClientCapabilities, InitializeParams,
    InitializeResult, InitializedParams, WorkspaceFolder,
};

#[derive(Debug)]
pub struct Client {
    proc: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    req_id: u32,
}

impl Client {
    pub fn new<S: AsRef<OsStr>>(program: S) -> std::io::Result<Self> {
        let mut proc = Command::new(program)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        Ok(Self {
            stdin: proc.stdin.take().unwrap(),
            stdout: BufReader::new(proc.stdout.take().unwrap()),
            proc,
            req_id: 1,
        })
    }

    fn req_id(&mut self) -> u32 {
        let id = self.req_id;
        self.req_id += 1;
        id
    }

    fn build_msg(content: impl OutboundMessage) -> serde_json::Result<String> {
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

    pub async fn recv_msg(&mut self) -> std::io::Result<Vec<u8>> {
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

    pub async fn request<R: RequestTrait>(&mut self, params: R::Params) -> Result<R::Result> {
        let id = self.req_id();
        let content = Request::<R> { id, params };

        let msg = Self::build_msg(content).unwrap();
        self.send_msg(msg.as_bytes())
            .await
            .map_err(Error::SendMsg)?;

        let r_msg = self.recv_msg().await.map_err(Error::RecvMsg)?;
        let response: Response<R::Result> = serde_json::from_slice(&r_msg).expect("bad data");

        assert_eq!(response.id, id);
        match (response.result, response.error) {
            (Some(r), None) => Ok(r),
            (None, Some(e)) => Err(Error::Lsp(e)),
            (Some(_), Some(_)) => Err(ProtocolViolation::BothResultAndResponse.into()),
            (None, None) => Err(ProtocolViolation::NeitherResultNorResponse.into()),
        }
    }

    pub async fn notify<N: NotificationTrait>(&mut self, params: N::Params) -> Result<()> {
        let content = Notification::<N> { params };
        let msg = Self::build_msg(content).unwrap();
        self.send_msg(msg.as_bytes())
            .await
            .map_err(Error::SendMsg)?;
        Ok(())
    }

    pub async fn initialize(
        &mut self,
        cwd: &str,
        capabilities: ClientCapabilities,
    ) -> Result<InitializeResult> {
        let uri = lsp_types::Url::from_directory_path(cwd).unwrap();
        let folder = WorkspaceFolder {
            uri: uri.clone(),
            name: cwd.into(),
        };

        #[allow(deprecated)]
        let params = InitializeParams {
            process_id: Some(std::process::id()),
            root_path: Some(cwd.into()),
            root_uri: Some(uri),
            initialization_options: None,
            capabilities,
            trace: None,
            workspace_folders: Some(vec![folder]),
            client_info: None,
            locale: None,
        };

        let res = self.request::<Initialize>(params).await?;
        self.notify::<Initialized>(InitializedParams {}).await?;
        Ok(res)
    }
}
