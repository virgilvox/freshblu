# syntax=docker/dockerfile:1

# Stage 1: Build
FROM rust:1.75-bookworm AS builder

WORKDIR /build

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
COPY crates/ironblu-core/Cargo.toml crates/ironblu-core/
COPY crates/ironblu-store/Cargo.toml crates/ironblu-store/
COPY crates/ironblu-server/Cargo.toml crates/ironblu-server/
COPY crates/ironblu-cli/Cargo.toml crates/ironblu-cli/

# Trick: create stub lib.rs files so cargo can download deps
RUN for crate in ironblu-core ironblu-store ironblu-server ironblu-cli; do \
    mkdir -p crates/$crate/src && echo "fn main(){}" > crates/$crate/src/main.rs && \
    touch crates/$crate/src/lib.rs; done

# Skip ironblu-wasm in server build (wasm32 target not needed in Docker)
RUN sed -i 's|"crates/ironblu-wasm",||' Cargo.toml

RUN cargo build --release --bin ironblu-server --bin ironblu 2>/dev/null || true

# Now copy real source
COPY crates/ crates/

# Build for real
RUN cargo build --release --bin ironblu-server --bin ironblu

# Stage 2: Runtime - minimal distroless image
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /build/target/release/ironblu-server /usr/local/bin/ironblu-server
COPY --from=builder /build/target/release/ironblu /usr/local/bin/ironblu

# Create data directory
RUN mkdir -p /data

# Default config via environment
ENV IRONBLU_HTTP_PORT=3000
ENV IRONBLU_MQTT_PORT=1883
ENV DATABASE_URL=sqlite:/data/ironblu.db
ENV RUST_LOG=info

EXPOSE 3000
EXPOSE 1883

VOLUME ["/data"]

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:3000/status || exit 1

CMD ["ironblu-server"]
