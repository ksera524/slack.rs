# Build stage
FROM rust:1.85.0-slim as builder

WORKDIR /usr/src/app

# Install dependencies needed for building
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy manifest files for dependency caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy the actual source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/slack /app/slack

# Copy necessary config files if needed
# COPY --from=builder /usr/src/app/config /app/config

# Expose the port your application listens on
EXPOSE 3000

# Set environment variables (customize as needed)
ENV RUST_LOG=slack=info,hyper=error,hyper_util=error,reqwest=error,axum::serve=info

# Run the application
CMD ["./slack"]