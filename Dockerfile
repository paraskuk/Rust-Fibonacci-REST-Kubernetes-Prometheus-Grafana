# Use the official Rust image as a base for building
FROM rust:1.93 AS builder

# Set a working directory for building
WORKDIR /usr/src/app

# Copy all source code into /usr/src/app
COPY . .

# Build in release mode
RUN cargo build --release

# Use a minimal base image
FROM debian:stable-slim

# Create a non-root user and group
RUN groupadd -r fibonacci && useradd -r -g fibonacci fibonacci

# Install minimal dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary
COPY --from=builder /usr/src/app/target/release/fibonacci /usr/local/bin/fibonacci

# Create directories and set permissions
RUN mkdir -p /usr/src/app/static /var/log && \
    chown -R fibonacci:fibonacci /usr/src/app /var/log && \
    chmod 755 /usr/local/bin/fibonacci

# Copy the static directory
COPY --chown=fibonacci:fibonacci static /usr/src/app/static

# Switch to non-root user
USER fibonacci

# Expose the Actix port
EXPOSE 8080

# Run the binary
ENTRYPOINT ["/usr/local/bin/fibonacci"]
