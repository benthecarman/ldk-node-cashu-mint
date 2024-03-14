use core::error;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Ok;
use bitcoin::{amount, Amount};
use ldk_node::bitcoin::Network;
use ldk_node::lightning_invoice::Bolt11Invoice;
use ldk_node::payment::PaymentStatus;
use ldk_node::{default_config, Builder, Node};
use mokshamint::lightning::Lightning;
use mokshamint::model::{CreateInvoiceResult, PayInvoiceResult};

struct LdkBackend {
    node: Node,
}

impl Lightning for LdkBackend {
    async fn is_invoice_paid(&self, invoice: String) -> Result<bool, Box<dyn error::Error>> {
        let invoice = self.decode_invoice(invoice).await?;
        let payment_details = self.node.payment(invoice.payment_hash()).unwrap();
        match payment_details.status {
            PaymentStatus::Succeeded => Ok(true),
            PaymentStatus::Pending => Ok(false),
            PaymentStatus::Failed => Ok(false),
        }
    }

    async fn create_invoice(
        &self,
        amount: u64,
    ) -> Result<CreateInvoiceResult, Box<dyn error::Error>> {
        let invoice = self
            .node
            .bolt11_payment()
            .receive(amount * 1000, "bla bla", expiry_secs)?;

        let result = CreateInvoiceResult {
            payment_hash: invoice.payment_hash(),
            payment_request: invoice.to_string(),
        };

        Ok(result)
    }

    async fn pay_invoice(
        &self,
        payment_request: String,
    ) -> Result<PayInvoiceResult, Box<dyn error::Error>> {
        let invoice = self.decode_invoice(payment_request).await?;
        let payment_result = self.node.bolt11_payment().send(invoice)?;

        let invoice_result = PayInvoiceResult {
            payment_hash: payment_result.to_string(),
            total_fees: 1,
        };

        Ok(invoice_result)
    }

    async fn decode_invoice(
        &self,
        payment_request: String,
    ) -> Result<Bolt11Invoice, Box<dyn error::Error>> {
        Bolt11Invoice::from_str(&payment_request)
    }
}

pub fn start_node() {
    let mut config = default_config();
    config.network = Network::Signet;

    let mut builder = Builder::from_config(config);
    builder.set_esplora_server("https://mutinynet.com/api/".to_string());

    let node = Arc::new(builder.build().unwrap());
    node.start().unwrap();

    let event_node = Arc::clone(&node);
    std::thread::spawn(move || loop {
        let event = event_node.wait_next_event();
    });

    // get address
    // send on-chain funds to address (get from mutinynet)

    // open channel (get node from mutinynet faucet)

    // send payment

    println!("Node ID: {}", node.node_id());
    println!("Address: {}", node.onchain_payment().new_address().unwrap());

    println!("Channels: {:?}", node.list_channels());
    println!("Node ID: {:?}", node.list_payments());

    node.stop().unwrap();
}
