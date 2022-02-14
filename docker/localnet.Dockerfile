FROM lukemathwalker/cargo-chef:latest-rust-latest AS chef
WORKDIR /app
RUN apt-get update -y \
        && apt-get install -y cmake pkg-config libssl-dev git clang

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin battlemon_indexer

FROM debian:bullseye-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends \
      openssl \
      ca-certificates \
      jq \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/battlemon_indexer /app/scripts/entry_point.sh ./
COPY --from=builder /app/configs/local_config.yaml ./config.yaml
ENV NEAR_HOME=/near
RUN  ./battlemon_indexer init \
    && sed -i 's/"tracked_shards": \[\]/"tracked_shards": [0]/' /near/config.json \
    && mkdir /near-configs \
    && cp /near/*.json /near-configs \
    && chmod +x entry_point.sh
VOLUME near-configs
ENTRYPOINT ["./entry_point.sh"]