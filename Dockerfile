FROM rust:1.95-slim-trixie AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src src
COPY web web
RUN cargo build --release --locked

FROM debian:trixie-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/honey-web /src/target/release/honey-stats \
     /src/target/release/honey-wordlists /src/target/release/honey-bloom \
     /src/target/release/honey-import /usr/local/bin/
COPY scripts/stats-gen.sh scripts/wordlist-gen.sh /usr/local/bin/
