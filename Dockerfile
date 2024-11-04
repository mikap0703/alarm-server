# Stage 1: Build dependencies and project
FROM rust:latest AS build

# Create an empty shell project
RUN USER=root cargo new --bin alarm-server
WORKDIR /alarm-server

# Copy manifests to leverage Docker's caching for dependencies
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

# Build dependencies only (no source code copied yet)
RUN cargo build --release
RUN rm -rf src

# Copy source files separately after caching dependencies
COPY ./src ./src

# Final build for the actual binary
RUN cargo build --release

# Stage 2: Final runtime image
FROM debian:bookworm-slim

# Set the working directory inside the container
WORKDIR /app

# Copy the compiled binary from the build stage
COPY --from=build /alarm-server/target/release/alarm-server /app/alarm-server

# Set the startup command to run your binary
CMD ["./alarm-server"]
