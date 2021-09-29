# Install locally

1. Install rust
```
curl https://sh.rustup.rs -sSf > sh.rustup.rs
sh sh.rustup.rs -y
```

2. Setup rust
```
RUN /root/.cargo/bin/rustup default stable
RUN /root/.cargo/bin/rustup update
RUN /root/.cargo/bin/rustup update nightly
RUN /root/.cargo/bin/rustup target add wasm32-unknown-unknown --toolchain nightly
```

3. Git clone substrate repo at the same root as this repo (side by side)
Current compatible tag is `monthly-2021-09`
```
git clone -b monthly-2021-09 https://github.com/paritytech/substrate.git
```

4. Compile it
Should take 15 minutes or more, be patient and hope for the best.
```
cargo build
```


# Run locally for development
```
./target/debug/wika-node --tmp --dev -lOWNERS=debug -lLIKE=debug
```
Optionally add Alice's private keys to the owner pallet
to enable offchain transactions.
```
curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d "@./dev_keys/alice_ownr.json"
```


# Start a test node
```
nohup ./target/release/wika-node \
    --public-addr /ip4/$NODE_IP \
    --base-path /var/db_wika/$NODE_NAME \
    --validator \
    --telemetry-url 'wss://telemetry.polkadot.io/submit/ 0' \
    --name $NODE_NAME \
    --chain $CHAIN_NAME \
    --port $CHAIN_PORT \
    --ws-port $WS_PORT \
    --rpc-port $RPC_PORT \
    $BOOT_NODE_OPTION \
    >/var/log/wikanode.log  &
```
