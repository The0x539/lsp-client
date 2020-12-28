use lsp_client::capabilities::caps;
use lsp_client::client::Client;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let mut client = Client::new("rust-analyzer")?;

    let cwd = std::env::current_dir()?.canonicalize()?;
    let res = client
        .initialize(cwd.to_str().unwrap(), caps())
        .await
        .unwrap();

    println!("{}", serde_json::to_string_pretty(&res)?);
    client.shutdown().await?;
    client.exit().await?;
    Ok(())
}
