FROM debian:10-slim

# Install dependencies
RUN apt update
RUN apt install -y git clang curl libssl-dev llvm libudev-dev

# Install rust
RUN mkdir install
WORKDIR install
RUN curl https://sh.rustup.rs -sSf > sh.rustup.rs
RUN sh sh.rustup.rs -y
RUN /root/.cargo/bin/rustup default stable
RUN /root/.cargo/bin/rustup update
RUN /root/.cargo/bin/rustup update nightly
RUN /root/.cargo/bin/rustup target add wasm32-unknown-unknown --toolchain nightly


# Checkout substrate
RUN git clone -b v3.0.0 https://github.com/paritytech/substrate.git

# Copy and compile wika project
COPY ./libs wika_node/libs
COPY ./node wika_node/node
COPY ./pallets wika_node/pallets
COPY ./runtime wika_node/runtime
COPY ./Cargo.toml wika_node/
WORKDIR wika_node
RUN /root/.cargo/bin/cargo build --release
RUN mv target/release /wika_release

# Back to root directory
WORKDIR /

# Start the blockchain
CMD /wika_release/wika-node \
    --base-path /tmp/node_data \
    --validator \
    --telemetry-url 'wss://telemetry.polkadot.io/submit/ 0' \
    --name $NODE_NAME \
    --chain $CHAIN_NAME \
    --port 30333 \
    --ws-port 9945 \
    --rpc-port 9933 \



