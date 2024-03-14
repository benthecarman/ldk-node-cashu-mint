use crate::config::Config;
use crate::ldk::LdkBackend;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Extension, Router};
use bitcoin::secp256k1::PublicKey;
use clap::Parser;
use ldk_node::lightning::ln::msgs::SocketAddress;
use ldk_node::lightning::util::logger::Level;
use mokshamint::config::{DatabaseConfig, LightningFeeConfig};
use mokshamint::lightning::Lightning;
use mokshamint::mint::MintBuilder;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use tokio::time::sleep;

use axum::Json;
use serde_json::{json, Value};
use std::collections::HashMap;

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

    let trusted_node = PublicKey::from_str(&config.trusted_node)?;

    let node_cfg = ldk_node::Config {
        network: config.network(),
        storage_dir_path: data_dir,
        log_level: Level::Trace,
        trusted_peers_0conf: vec![trusted_node],
        ..Default::default()
    };

    let mut builder = ldk_node::Builder::from_config(node_cfg);
    builder.set_esplora_server(config.esplora_url());
    builder.set_liquidity_source_lsps2(
        SocketAddress::from_str(&config.trusted_socket_addr).expect("Invalid socket address"),
        trusted_node,
        Some(config.lsps_token),
    );

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
        .build(Some(ldk_backend.clone()))
        .await?;

    tokio::spawn(async move {
        loop {
            if let Err(e) = mokshamint::server::run_server(mint.clone()).await {
                eprintln!("Error running mint: {}", e);
            }
        }
    });

    let state = State {
        ldk: ldk_backend.clone(),
    };
    tokio::spawn(async move {
        loop {
            let ln_router = Router::new()
                .route("/invoice", get(get_invoice))
                .layer(Extension(state.clone()));
            let listener = tokio::net::TcpListener::bind(listening_addr.clone())
                .await
                .unwrap();
            axum::serve(listener, ln_router).await.unwrap();
        }
    });

    println!("Hello, welcome to Nostr world!");
    let _ = nostr::nostr_listener().await;
    println!("Bye!");

    sleep(std::time::Duration::from_secs(5)).await;

    ldk_backend.node.stop()?;

    Ok(())
}

#[derive(Clone)]
pub struct State {
    pub ldk: Arc<LdkBackend>,
}

pub async fn get_invoice(
    Query(params): Query<HashMap<String, String>>,
    Extension(state): Extension<State>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let amount_sats = params
        .get("amount")
        .and_then(|a| a.parse::<u64>().ok())
        .unwrap();
    let invoice_result = state.ldk.create_invoice(amount_sats).await.unwrap();
    Ok(Json(json!(invoice_result)))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelInfo {
    pub channel_id: String,
    pub counterparty_node_id: PublicKey,
    pub channel_value_sats: u64,
    pub outbound_capacity_sat: u64,
    pub inbound_capacity_sat: u64,
    pub is_channel_ready: bool,
}

pub async fn list_channels(
    Extension(state): Extension<State>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let channels_details = state.ldk.node.list_channels();

    let channels: Vec<ChannelInfo> = channels_details
        .iter()
        .map(|channel| ChannelInfo {
            channel_id: channel.channel_id.to_string(),
            counterparty_node_id: channel.counterparty_node_id,
            channel_value_sats: channel.channel_value_sats,
            outbound_capacity_sat: channel.outbound_capacity_msat * 1000,
            inbound_capacity_sat: channel.inbound_capacity_msat * 1000,
            is_channel_ready: channel.is_channel_ready,
        })
        .collect();

    let json = serde_json::to_string(&channels).unwrap();
    Ok(Json(json!(json)))
}
