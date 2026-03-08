# syntax=docker/dockerfile:1

# Stage 1: Build
FROM rust:1-bookworm AS builder

WORKDIR /build

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
COPY crates/freshblu-core/Cargo.toml crates/freshblu-core/
COPY crates/freshblu-store/Cargo.toml crates/freshblu-store/
COPY crates/freshblu-server/Cargo.toml crates/freshblu-server/
COPY crates/freshblu-cli/Cargo.toml crates/freshblu-cli/

# Trick: create stub lib.rs files so cargo can download deps
RUN for crate in freshblu-core freshblu-store freshblu-server freshblu-cli; do \
    mkdir -p crates/$crate/src && echo "fn main(){}" > crates/$crate/src/main.rs && \
    touch crates/$crate/src/lib.rs; done

# Skip freshblu-wasm in server build (wasm32 target not needed in Docker)
RUN sed -i 's|"crates/freshblu-wasm",||' Cargo.toml

RUN cargo build --release --bin freshblu-server --bin freshblu 2>/dev/null || true

# Now copy real source
COPY crates/ crates/

# Build for real
RUN cargo build --release --bin freshblu-server --bin freshblu

# Stage 2: Runtime - minimal distroless image
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /build/target/release/freshblu-server /usr/local/bin/freshblu-server
COPY --from=builder /build/target/release/freshblu /usr/local/bin/freshblu

# Create data directory
RUN mkdir -p /data

# Default config via environment
ENV FRESHBLU_HTTP_PORT=3000
ENV FRESHBLU_MQTT_PORT=1883
ENV DATABASE_URL=sqlite:/data/freshblu.db
ENV RUST_LOG=info

EXPOSE 3000
EXPOSE 1883

VOLUME ["/data"]

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:3000/status || exit 1

CMD ["freshblu-server"]
