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
    let _x = format!("{}", serde_json::to_string_pretty(&res)?);
    println!("got here");
    Ok(())
}
