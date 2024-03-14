use std::str::FromStr;
use std::sync::Arc;

use async_trait::async_trait;
use bitcoin::hashes::Hash;
use ldk_node::bitcoin::Network;
use ldk_node::lightning::ln::channelmanager::PaymentId;
use ldk_node::payment::PaymentStatus;
use ldk_node::{default_config, Builder, Node};
use lightning_invoice_26::Bolt11Invoice;
use mokshamint::error::MokshaMintError;
use mokshamint::lightning::error::LightningError;
use mokshamint::lightning::Lightning;
use mokshamint::model::{CreateInvoiceResult, PayInvoiceResult};

struct LdkBackend {
    node: Node,
}

#[async_trait]
impl Lightning for LdkBackend {
    async fn is_invoice_paid(&self, invoice: String) -> Result<bool, MokshaMintError> {
        let invoice: Bolt11Invoice = self.decode_invoice(invoice).await?;
        let id_bytes: [u8; 32] = invoice
            .payment_hash()
            .to_vec()
            .try_into()
            .expect("always 32 bytes");
        let id = PaymentId(id_bytes);
        let payment_details = self.node.payment(&id);
        match payment_details.map(|p| p.status) {
            Some(PaymentStatus::Succeeded) => Ok(true),
            Some(PaymentStatus::Pending) => Ok(false),
            Some(PaymentStatus::Failed) => Ok(false),
            None => Ok(false),
        }
    }

    async fn create_invoice(&self, amount: u64) -> Result<CreateInvoiceResult, MokshaMintError> {
        let invoice = self
            .node
            .bolt11_payment()
            .receive(amount * 1000, "bla bla", 86_400)
            .map_err(|_| MokshaMintError::InvoiceNotPaidYet)?;

        let result = CreateInvoiceResult {
            payment_hash: invoice.payment_hash().to_byte_array().to_vec(),
            payment_request: invoice.to_string(),
        };

        Ok(result)
    }

    async fn pay_invoice(
        &self,
        payment_request: String,
    ) -> Result<PayInvoiceResult, MokshaMintError> {
        let invoice =
            ldk_node::lightning_invoice::Bolt11Invoice::from_str(&payment_request).unwrap();
        let payment_result = self.node.bolt11_payment().send(&invoice).map_err(|_| {
            MokshaMintError::PayInvoice(
                "Failed to pay invoice".to_string(),
                LightningError::PaymentFailed,
            )
        })?;

        // todo do we need wait for payment success?

        let invoice_result = PayInvoiceResult {
            payment_hash: payment_result.to_string(),
            total_fees: 1,
        };

        Ok(invoice_result)
    }

    async fn decode_invoice(
        &self,
        payment_request: String,
    ) -> Result<Bolt11Invoice, MokshaMintError> {
        Ok(Bolt11Invoice::from_str(&payment_request).map_err(|e| {
            MokshaMintError::DecodeInvoice(payment_request, e)
        })?)
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
