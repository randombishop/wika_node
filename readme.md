# Run a docker test node

1. Build and run your own docker image using docker-compose file. Build time could take up to 60 min.

    ```sh
    git clone https://github.com/randombishop/wika_node.git
    docker-compose -p my_wika_node -f docker-compose.yaml up --build
    ```

2. Pull and run latest image of wika test node using docker-compose file

    ```sh
    git clone https://github.com/randombishop/wika_node.git
    docker-compose -p my_wika_node -f docker-compose.yaml up
    ```

# Install locally

1. Follow substrate documentation to install dependencies (do #1 only)
<https://substrate.dev/docs/en/knowledgebase/getting-started/>
2. For example, in apt world

    ```sh
    sudo apt update
    sudo apt install -y git clang curl libssl-dev llvm libudev-dev
    ```

3. Install and setup rust

    ```sh
    curl https://sh.rustup.rs -sSf | sh
    source ~/.cargo/env
    rustup default stable
    rustup update
    rustup update nightly
    rustup target add wasm32-unknown-unknown --toolchain nightly
    ```

4. Git clone substrate

    Current substrate repo compatible tag is `monthly-2021-08`

    ```sh
    git clone https://github.com/paritytech/substrate.git
    cd substrate
    git checkout monthly-2021-08
    cd ..
    ```

5. Git clone wika-node side by side with substrate

    ```sh
    git clone https://github.com/randombishop/wika_node.git
    ```

6. Compile it
Should take 20 minutes to 1 hour depending on your number of CPUs, also note that a minimum of 4Gb of RAM is required here.

    ```sh
    cd wika_node
    cargo build
    or
    cargo build --release
    ```

# Run locally for development

```sh
./target/debug/wika-node --tmp --dev -lOWNERS=debug -lLIKE=debug
or
./target/release/wika-node --tmp --dev -lOWNERS=debug -lLIKE=debug
```

# Enabling offchain worker with Alice as a verifier

1. Add Alice's private keys to the owner pallet.

    ```sh
    curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d "@./dev_keys/alice_ownr.json"
    ```

2. Use the owners pallet addVerifier transaction to add Alice public key as a verifier.

# Generating a chain spec file

(<https://substrate.dev/docs/en/tutorials/start-a-private-network/customspec>)

```sh
.target/release/wika-node  --chain test build-spec > specfile.json
```

# Start a test node and join test net

(<https://substrate.dev/docs/en/tutorials/start-a-private-network/customchain>)

```sh
./target/release/wika-node \
    --public-addr /ip4/x.x.x.x \
    --base-path /var/db_wika/test1 \
    --telemetry-url 'wss://telemetry.polkadot.io/submit/ 0' \
    --name xyz1 \
    --chain specfile.json \
    --bootnodes /ip4/z.z.z.z/tcp/30334/p2p/xyz
```
