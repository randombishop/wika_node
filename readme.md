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

3. Git clone substrate repo and wika_node repo side by side
Current substrate repo compatible tag is `0a785f1221f5d143ae4487147183d66bad0f9837`
```
git clone -b 0a785f1221f5d143ae4487147183d66bad0f9837 https://github.com/paritytech/substrate.git
```

4. Compile it
Should take 20 minutes or more, be patient and hope for the best.
```
cargo build --release
```


# Run locally for development
```
./target/release/wika-node --tmp --dev -lOWNERS=debug -lLIKE=debug
```

# Enabling offchain worker with Alice as a verifier
1. Add Alice's private keys to the owner pallet.
```
curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d "@./dev_keys/alice_ownr.json"
```
2. Use the owners pallet addVerifier transaction to add Alice public key as a verifier.


# Start a test node
```
./target/release/wika-node \
    --public-addr /ip4/x.x.x.x \
    --base-path /var/db_wika/test1 \
    --validator \
    --telemetry-url 'wss://telemetry.polkadot.io/submit/ 0' \
    --name test1 \
    --chain test \
    --port 30334 \
    --ws-port 9944 \
    --rpc-port 9934 \
    --bootnodes /ip4/z.z.z.z/tcp/30334/p2p/xyz
```
