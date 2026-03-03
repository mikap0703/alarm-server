# Stage 1: Build dependencies and project
# Change 'latest' to 'bookworm' to match your runtime OS version
FROM rust:bookworm AS builder

RUN apt update && apt install -y libudev-dev pkg-config

WORKDIR /alarm-server

# Copy the Cargo manifests
COPY Cargo.toml Cargo.lock ./

# Build dependencies only
RUN cargo build --release || true

# Copy the source code
COPY ./src ./src

# Final build
RUN cargo build --release

# Stage 2: Runtime environment
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    libudev1 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
RUN mkdir -p /app/config

# Copy the binary
COPY --from=builder /alarm-server/target/release/alarm-server /app/alarm-server
RUN chmod +x /app/alarm-server

CMD ["./alarm-server"]