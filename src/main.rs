use crate::config::Config;
use crate::ldk::LdkBackend;
use clap::Parser;
use ldk_node::lightning::util::logger::Level;
use mokshamint::config::{DatabaseConfig, LightningFeeConfig};
use mokshamint::mint::MintBuilder;
use std::sync::Arc;
use tokio::time::sleep;

mod cashu;
mod config;
mod ldk;
mod nostr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    // pretty_env_logger::try_init()?;

    let config = Config::parse();

    println!("Starting!");
    println!("Data dir: {}", config.data_dir);
    println!("Network: {}", config.network());
    println!("Esplora: {}", config.esplora_url());
    println!("Relays: {:?}", config.relay);

    let listening_addr = format!("{}:{}", config.bind, config.port);
    println!("Listening on {}", listening_addr);

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

    println!("Node started!");
    println!("Node ID: {}", node.node_id());

    let event_node = Arc::clone(&node);
    std::thread::spawn(move || loop {
        let event = event_node.wait_next_event();
        println!("GOT NEW EVENT: {event:?}");
        println!("Channels: {:?}", event_node.list_channels());
        println!("Payments: {:?}", event_node.list_payments());
        event_node.event_handled();
    });

    let ldk_backend = Arc::new(LdkBackend { node });

    let db_config = DatabaseConfig {
        db_url: config.pg_url,
        max_connections: 10,
    };

    let fees = LightningFeeConfig {
        fee_percent: 1.0,
        fee_reserve_min: 4000,
    };

    let private_key = ldk_backend
        .node
        .sign_message(b"signing this message to create a private key")?;
    let mint = MintBuilder::new()
        .with_private_key(private_key)
        .with_db(Some(db_config))
        .with_fee(Some(fees))
        .build(Some(ldk_backend))
        .await?;

    mokshamint::server::run_server(mint).await?;

    println!("Hello, welcome to Nostr world!");
    let _ = nostr::nostr_listener().await;
    println!("Bye!");

    sleep(std::time::Duration::from_secs(5)).await;

    //ldk_backend.node.stop()?;

    Ok(())
}
