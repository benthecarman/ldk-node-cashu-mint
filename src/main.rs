use crate::config::Config;
use clap::Parser;
use log::info;

mod cashu;
mod config;
mod ldk;
mod nostr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    pretty_env_logger::try_init()?;

    let config = Config::parse();

    info!("Starting!");
    info!("Data dir: {}", config.data_dir);
    info!("Seed path: {}", config.seed_path());
    info!("Network: {}", config.network());
    info!("Esplora: {}", config.esplora_url());
    info!("Relays: {:?}", config.relay);

    let listening_addr = format!("{}:{}", config.bind, config.port);
    info!("Listening on {}", listening_addr);

    Ok(())
}
