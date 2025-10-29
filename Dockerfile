# syntax=docker/dockerfile:1.7-labs
FROM ghcr.io/dhayes/rust-musl-builder:30d1a75 AS builder
WORKDIR /app

# Copy manifests first to cache deps
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && printf "fn main() {}\n" > src/main.rs && \
    cargo build --release

# Real sources
RUN rm -rf src
COPY src ./src
RUN cargo build --release

# Optional: strip
RUN strip /app/target/x86_64-unknown-linux-musl/release/authserver

# Minimal runtime
FROM alpine:3.20
RUN apk add --no-cache ca-certificates
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/authserver /usr/local/bin/authserver
ENTRYPOINT ["/usr/local/bin/authserver"]
# EXPOSE 8080

