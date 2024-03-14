
run:
    RUST_LOG="ldk_node_cashu_mint=debug,ldk_node=debug,lightning=debug" cargo run -- --relay "wss://relay.damus.io" --pg-url "postgres://username:password@localhost:5432/ldk_cashu" --trusted-node "0371d6fd7d75de2d0372d03ea00e8bacdacb50c27d0eaea0a76a0622eff1f5ef2b" --trusted-socket-addr "44.219.111.31:39735" --lsps-token "T2MF3ZU5"
