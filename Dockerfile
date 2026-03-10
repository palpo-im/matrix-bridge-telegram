FROM rust:1.93-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src/ ./src/

RUN cargo build --release --features "postgres,sqlite"

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/matrix-bridge-telegram /usr/local/bin/

RUN mkdir -p /data
WORKDIR /data

ENV CONFIG_PATH=/data/config.yaml
ENV REGISTRATION_PATH=/data/telegram-registration.yaml

EXPOSE 29317

CMD ["matrix-bridge-telegram"]
