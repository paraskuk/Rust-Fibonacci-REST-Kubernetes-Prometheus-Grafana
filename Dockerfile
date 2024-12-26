# Use the official Rust image as a base for building
FROM rust:1.80.1 AS builder

# Set a working directory for building
WORKDIR /usr/src/app

# Copy all source code into /usr/src/app
COPY . .

# Build in release mode
RUN cargo build --release

# Use a minimal base image
FROM debian:stable-slim

# Curl and Nano installation
RUN apt-get update && apt-get install -y curl nano && rm -rf /var/lib/apt/lists/*

# We do NOT set WORKDIR, so we'll rely on absolute paths in the binary.

# Copy the compiled binary
COPY --from=builder /usr/src/app/target/release/fibonacci /usr/local/bin/fibonacci

# Copy the static directory into /usr/src/app/static
# so that /usr/src/app/static/index.html exists
RUN mkdir -p /usr/src/app/static
COPY static /usr/src/app/static

# If you need to adjust file permissions (optional but often recommended)
RUN chmod 755 /usr/local/bin/fibonacci && \
    chmod -R 755 /usr/src/app/static

# Expose the Actix port
EXPOSE 8080

# Run the binary
ENTRYPOINT ["/usr/local/bin/fibonacci"]
