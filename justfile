
run:
    RUST_LOG="ldk_node_cashu_mint=debug,ldk_node=debug,lightning=debug" cargo run -- -r wss://relay.damus.io --pg-url "todo"
