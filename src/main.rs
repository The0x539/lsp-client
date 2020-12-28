use lsp_client::capabilities::caps;
use lsp_client::client::Client;

use std::time::Duration;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let mut client = Client::new("rust-analyzer")?;

    let cwd = std::env::current_dir()?.canonicalize()?;

    let _res = client.initialize(cwd.to_str().unwrap(), caps()).await?;

    //println!("{}", serde_json::to_string_pretty(&res)?);

    let params = lsp_types::SemanticTokensParams {
        work_done_progress_params: lsp_types::WorkDoneProgressParams {
            work_done_token: None,
        },
        partial_result_params: lsp_types::PartialResultParams {
            partial_result_token: None,
        },
        text_document: lsp_types::TextDocumentIdentifier {
            uri: lsp_types::Url::from_file_path(format!("{}/src/main.rs", cwd.display())).unwrap(),
        },
    };

    std::thread::sleep(Duration::from_secs(1));

    let semantic = client
        .request::<lsp_types::request::SemanticTokensFullRequest>(params)
        .await?
        .expect("tokens pls");

    let data = match semantic {
        lsp_types::SemanticTokensResult::Tokens(t) => t.data,
        lsp_types::SemanticTokensResult::Partial(t) => t.data,
    };

    let source_code = tokio::fs::read_to_string("src/main.rs").await?;
    let mut lines = source_code
        .lines()
        .map(|s| s.encode_utf16().collect::<Vec<_>>());

    let mut line = lines.next().unwrap();
    let mut pos = 0;
    for token in data {
        for _ in 0..token.delta_line {
            line = lines.next().unwrap();
            pos = 0;
        }
        pos += token.delta_start;
        let start = pos as usize;
        let end = start + token.length as usize;
        let text = &line[start..end];
        print!("{} {}", String::from_utf16_lossy(text), token.token_type);
        if token.token_modifiers_bitset != 0 {
            print!(" {}", token.token_modifiers_bitset);
        }
        println!();
    }

    client.shutdown().await?;
    client.exit().await?;
    Ok(())
}
