use nostr_sdk::prelude::*;

const BECH32_SK: &str = "nsec1yk3fasxr4k02jnexhv3q39wgwazqt3rmxm24lx2u8uey3w4svppqpkcvzn";

pub async fn nostr_listener() -> Result<()> {
    // tracing_subscriber::fmt::init();

    let secret_key = SecretKey::from_bech32(BECH32_SK)?;
    let keys = Keys::new(secret_key);
    let public_key = keys.public_key();
    println!("npub: {:?}", public_key.to_bech32());

    let opts = Options::new().wait_for_send(false);
    let client = ClientBuilder::new().opts(opts).build();

    client.add_relay("wss://nostr.oxtr.dev").await?;
    client.add_relay("wss://relay.damus.io").await?;
    client.add_relay("wss://nostr.openchain.fr").await?;

    client.connect().await;

    let subscription_1 = Filter::new()
        .author(public_key)
        .kinds([Kind::Metadata, Kind::EncryptedDirectMessage])
        .since(Timestamp::now());

    let subscription_2 = Filter::new()
        .pubkey(public_key)
        .kinds([Kind::Metadata, Kind::EncryptedDirectMessage])
        .since(Timestamp::now());

    // Subscribe (auto generate subscription ID)
    // let opts = SubscribeAutoCloseOptions::default().filter(FilterOptions::ExitOnEOSE);
    let sub_id = client
        .subscribe(vec![subscription_1, subscription_2], None)
        .await;

    // Handle subscription notifications with `handle_notifications` method
    client
        .handle_notifications(|notification| async {
            if let RelayPoolNotification::Event {
                subscription_id,
                event,
                ..
            } = notification
            {
                // Check subscription ID
                if subscription_id == sub_id {
                    // Handle (ex. update specific UI)
                }

                // Check kind
                if event.kind() == Kind::EncryptedDirectMessage {
                    if let Ok(msg) =
                        nip04::decrypt(keys.secret_key()?, event.author_ref(), event.content())
                    {
                        println!("DM: {msg}");
                    } else {
                        eprintln!("Impossible to decrypt direct message");
                        // tracing::error!("Impossible to decrypt direct message");
                    }
                } else if event.kind() == Kind::TextNote {
                    println!("TextNote: {:?}", event);
                } else {
                    println!("{:?}", event);
                }
            }
            Ok(false) // Set to true to exit from the loop
        })
        .await?;

    Ok(())
}
