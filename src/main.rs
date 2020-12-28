use lsp_client::capabilities::caps;
use lsp_client::client::Client;

fn main() -> anyhow::Result<()> {
    let mut client = Client::new("rust-analyzer")?;

    let cwd = std::env::current_dir()?.canonicalize()?;
    let res = client.initialize(cwd.to_str().unwrap(), caps()).unwrap();
    let _x = format!("{}", serde_json::to_string_pretty(&res)?);
    println!("got here");

    loop {
        let msg = client.recv_msg()?;
        println!("{}", std::str::from_utf8(&msg).unwrap());
    }
}
