FROM rust:1.86-bookworm AS builder
WORKDIR /app
COPY backend-rust/Cargo.toml /app/Cargo.toml
COPY backend-rust/src /app/src
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/aegis-broker-backend ./aegis-broker-backend
EXPOSE 8080
CMD ["./aegis-broker-backend"]
