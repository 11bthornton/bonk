FROM rust:1.87 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock build.rs ./
COPY src/ src/
COPY static/ static/
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/bonk /usr/local/bin/bonk
ENV PORT=8080
EXPOSE 8080
CMD ["bonk", "serve"]
