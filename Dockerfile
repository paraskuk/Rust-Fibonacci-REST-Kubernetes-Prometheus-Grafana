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

# Install dependencies for log4rs if any (optional)

# Copy the compiled binary
COPY --from=builder /usr/src/app/target/release/fibonacci /usr/local/bin/fibonacci

# Copy the static directory
RUN mkdir -p /usr/src/app/static
COPY static /usr/src/app/static

# Ensure the log directory exists
RUN mkdir -p /var/log

# Set correct permissions
RUN chmod 755 /usr/local/bin/fibonacci && \
    chmod -R 755 /usr/src/app/static && \
    chmod -R 755 /var/log

# Expose the Actix port
EXPOSE 8080

# Run the binary
ENTRYPOINT ["/usr/local/bin/fibonacci"]
