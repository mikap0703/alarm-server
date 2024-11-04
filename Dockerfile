# Stage 1: Build dependencies and project
FROM rust:latest AS builder

# Set the working directory for the build
WORKDIR /alarm-server

# Copy the Cargo manifests to leverage Docker's cache for dependencies
COPY Cargo.toml Cargo.lock ./

# Build dependencies only (no source code copied yet)
RUN cargo build --release --manifest-path ./Cargo.toml || true

# Now copy the source code
COPY ./src ./src

# Final build for the actual binary with the full project source
RUN cargo build --release

# Stage 2: Runtime environment
FROM debian:bookworm-slim

# Install runtime dependencies, including libssl for OpenSSL support
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory inside the container
WORKDIR /app

# Copy the compiled binary from the build stage
COPY --from=builder /alarm-server/target/release/alarm-server /app/alarm-server

# Ensure the binary is executable
RUN chmod +x /app/alarm-server

# Set the startup command to run your binary
CMD ["./alarm-server"]
