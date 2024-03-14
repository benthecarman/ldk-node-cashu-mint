use std::str::FromStr;
use bitcoin::Network;
use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(version, author, about)]
/// An LDK node with a built-in cashu mint
pub struct Config {
	/// Location keys files
	#[clap(default_value = "./data", long)]
	pub data_dir: String,
	/// Postgres connection string
	#[clap(long)]
	pub pg_url: String,
	/// Relay to connect to, can be specified multiple times
	#[clap(short, long)]
	pub relay: Vec<String>,
	/// Node we trust a 0 conf channel from, their pubkey
	#[clap(long)]
	pub trusted_node: String,
	/// Node we trust a 0 conf channel from, their connection string
	#[clap(long)]
	pub trusted_socket_addr: String,
	/// Token for LSPS
	#[clap(long)]
	pub lsps_token: String,
	/// Bind address for webserver
	#[clap(default_value = "0.0.0.0", long)]
	pub bind: String,
	/// Port for webserver
	#[clap(default_value_t = 3000, long)]
	pub port: u16,
	/// Network
	#[clap(default_value = "signet", long)]
	pub network: String,
	/// Network
	#[clap(default_value = "https://mutinynet.com/api", long)]
	pub esplora: String,
}

impl Config {
	pub fn network(&self) -> Network {
		Network::from_str(&self.network).expect("Invalid network")
	}

	pub fn esplora_url(&self) -> String {
		self.esplora.clone()
	}
}
