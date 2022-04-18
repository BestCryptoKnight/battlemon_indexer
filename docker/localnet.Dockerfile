FROM lukemathwalker/cargo-chef:latest-rust-latest AS chef
WORKDIR /app
RUN apt-get update -y \
        && apt-get install -y cmake pkg-config libssl-dev git clang

FROM debian:bullseye-slim AS runtime
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends \
      openssl \
      ca-certificates \
      jq \
      cmake \
      pkg-config \
      libssl-dev \
      git \
      clang \
      curl \
      gnupg \
    && curl -sL https://deb.nodesource.com/setup_17.x  | bash - \
    && apt-get -y install nodejs \
    && npm install -g near-cli \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin battlemon_indexer

FROM runtime
WORKDIR /app

COPY --from=builder /app/target/release/battlemon_indexer /app/scripts/entry_point.sh ./
COPY --from=builder /app/configs/local_config.yaml ./config.yaml
COPY ./credentials /root/.aws/credentials
COPY ./battlemon.testnet.json /root/.near-credentials/testnet/battlemon.testnet.json
RUN chmod +x entry_point.sh

ENTRYPOINT ["./entry_point.sh"]