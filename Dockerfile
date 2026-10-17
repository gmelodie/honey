# syntax=docker/dockerfile:1
FROM rust:1.95-slim-trixie AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src src
COPY web web
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked \
    && mkdir /out \
    && cp target/release/honey-web target/release/honey-stats target/release/honey-wordlists \
          target/release/honey-bloom target/release/honey-import /out/

FROM debian:trixie-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /out/ /usr/local/bin/
COPY scripts/stats-gen.sh scripts/wordlist-gen.sh /usr/local/bin/
