use std::sync::Arc;
use crate::config::Config;
use crate::ldk::LdkBackend;
use clap::Parser;
use ldk_node::lightning::util::logger::Level;
use log::{debug, info};
use tokio::time::sleep;

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
    info!("Network: {}", config.network());
    info!("Esplora: {}", config.esplora_url());
    info!("Relays: {:?}", config.relay);

    let listening_addr = format!("{}:{}", config.bind, config.port);
    info!("Listening on {}", listening_addr);

    let data_dir = config.data_dir.clone();

    let node_cfg = ldk_node::Config {
        network: config.network(),
        storage_dir_path: data_dir,
        log_level: Level::Trace,
        ..Default::default()
    };

    let mut builder = ldk_node::Builder::from_config(node_cfg);
    builder.set_esplora_server(config.esplora_url());

    let node = Arc::new(builder.build()?);

    node.start()?;

    info!("Node started!");
    info!("Node ID: {}", node.node_id());

    let event_node = Arc::clone(&node);
    std::thread::spawn(move || loop {
        let event = event_node.wait_next_event();
        info!("GOT NEW EVENT: {event:?}");
        debug!("Channels: {:?}", event_node.list_channels());
        debug!("Payments: {:?}", event_node.list_payments());
        event_node.event_handled();
    });

    let ldk_backend = LdkBackend { node };

    println!("Hello, welcome to Nostr world!");
    let _ = nostr::nostr_listener().await;
    println!("Bye!");

    sleep(std::time::Duration::from_secs(5)).await;

    ldk_backend.node.stop()?;

    Ok(())
}
