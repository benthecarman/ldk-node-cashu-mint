
run:
    RUST_LOG="ldk_node_cashu_mint=debug,ldk_node=debug,lightning=debug" cargo run -- --relay "wss://relay.damus.io" --pg-url "postgres://username:password@localhost:5432/ldk_cashu"
