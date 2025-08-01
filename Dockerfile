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

# Build arguments for CPU compatibility
ARG RUSTFLAGS="-C target-cpu=x86-64 -C target-feature=-aes,-avx,-avx2"
ENV RUSTFLAGS=${RUSTFLAGS}

# Build the application with CPU compatibility
RUN echo "Building with RUSTFLAGS: $RUSTFLAGS" && \
    cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/slack /app/slack

# Expose the port your application listens on
EXPOSE 3000

RUN chmod +x /app/slack
# Run the application
CMD ["./slack"]