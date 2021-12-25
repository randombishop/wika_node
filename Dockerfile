FROM debian:stable-slim AS base

FROM base AS builder 
RUN apt -y update && apt -y install cmake pkg-config libssl-dev git build-essential clang libclang-dev curl
ENV PATH="/root/.cargo/bin:${PATH}"
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
    rustup default stable && \ 
    rustup update && \  
    rustup update nightly && \ 
    rustup target add wasm32-unknown-unknown --toolchain nightly  
RUN git clone --depth 1 --branch 'monthly-2021-08' https://github.com/paritytech/substrate.git && \      
    git clone --depth 1 https://github.com/randombishop/wika_node.git    
RUN cd wika_node && cargo build --release    

FROM base as final
WORKDIR /wika_node
COPY genesis_spec.json /wika_node
COPY --from=builder /wika_node/target/release/wika-node .  
 
ENTRYPOINT ["/bin/bash", "-c", "./wika-node --base-path /var/db_wika --chain genesis_spec.json --telemetry-url 'wss://telemetry.polkadot.io/submit/ 0' \"$@\"", "--"]
