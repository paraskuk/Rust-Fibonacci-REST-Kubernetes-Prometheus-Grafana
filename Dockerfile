# Use the official Rust image as a base
FROM rust:1.70 AS builder

# Set the working directory
WORKDIR /usr/src/app

# Copy the source code
COPY . .

# Normalize file permissions
RUN find . -type f -exec chmod 644 {} \; && \
    find . -type d -exec chmod 755 {} \;

# Build the application
RUN cargo build --release

# Use a minimal base image
FROM debian:stable-slim

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/fibonacci /usr/local/bin/fibonacci

# Set the correct permissions for the binary
RUN chmod 755 /usr/local/bin/fibonacci

# Set the entrypoint
ENTRYPOINT ["fibonacci"]