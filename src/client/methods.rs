use crate::error::Result;

use lsp_types::{
    notification::{Exit, Initialized},
    request::{Initialize, Shutdown},
    ClientCapabilities, InitializeParams, InitializeResult, InitializedParams, WorkspaceFolder,
};

use super::Client;

impl Client {
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

    pub async fn shutdown(&mut self) -> Result<()> {
        self.request::<Shutdown>(()).await
    }

    pub async fn exit(&mut self) -> Result<()> {
        self.notify::<Exit>(()).await
    }
}
