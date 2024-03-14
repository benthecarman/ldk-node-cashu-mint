use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use bitcoin::hashes::Hash;
use ldk_node::lightning::ln::channelmanager::PaymentId;
use ldk_node::payment::PaymentStatus;
use ldk_node::Node;
use lightning_invoice_26::Bolt11Invoice;
use mokshamint::error::MokshaMintError;
use mokshamint::lightning::error::LightningError;
use mokshamint::lightning::Lightning;
use mokshamint::model::{CreateInvoiceResult, PayInvoiceResult};
use tokio::time::sleep;

#[derive(Clone)]
pub struct LdkBackend {
    pub node: Arc<Node>,
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
        // if we don't have enough funds, get a JIT channel
        let invoice = if self
            .node
            .list_channels()
            .iter()
            .filter(|c| c.is_channel_ready)
            .map(|c| c.inbound_capacity_msat)
            .sum::<u64>()
            < amount * 1000
        {
            self.node
                .bolt11_payment()
                .receive_via_jit_channel(amount * 1000, "bla bla", 86_400, None)
                .map_err(|_| MokshaMintError::InvoiceNotPaidYet)?
        } else {
            self.node
                .bolt11_payment()
                .receive(amount * 1000, "bla bla", 86_400)
                .map_err(|_| MokshaMintError::InvoiceNotPaidYet)?
        };

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
        let payment_id = self.node.bolt11_payment().send(&invoice).map_err(|_| {
            MokshaMintError::PayInvoice(
                "Failed to pay invoice".to_string(),
                LightningError::PaymentFailed,
            )
        })?;

        // todo do we need wait for payment success?
        loop {
            sleep(Duration::from_millis(100)).await;
            match self.node.payment(&payment_id) {
                Some(payment) => match payment.status {
                    PaymentStatus::Succeeded => {
                        break;
                    }
                    PaymentStatus::Pending => {}
                    PaymentStatus::Failed => {
                        return Err(MokshaMintError::PayInvoice(
                            "Failed to pay invoice".to_string(),
                            LightningError::PaymentFailed,
                        ));
                    }
                },
                None => {
                    return Err(MokshaMintError::PayInvoice(
                        "Failed to pay invoice".to_string(),
                        LightningError::PaymentFailed,
                    ));
                }
            }
        }

        let invoice_result = PayInvoiceResult {
            payment_hash: invoice.payment_hash().to_string(),
            total_fees: 1,
        };

        Ok(invoice_result)
    }

    async fn decode_invoice(
        &self,
        payment_request: String,
    ) -> Result<Bolt11Invoice, MokshaMintError> {
        Ok(Bolt11Invoice::from_str(&payment_request)
            .map_err(|e| MokshaMintError::DecodeInvoice(payment_request, e))?)
    }
}
