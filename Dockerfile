# Build stage
FROM rust:1.88-slim AS builder

WORKDIR /usr/src/app

# Build static binary for scratch runtime
RUN apt-get update && \
    apt-get install -y --no-install-recommends musl-tools && \
    rm -rf /var/lib/apt/lists/* && \
    rustup target add x86_64-unknown-linux-musl

ARG RUSTFLAGS="-C target-cpu=x86-64 -C target-feature=-aes,-avx,-avx2"
ENV RUSTFLAGS=${RUSTFLAGS}
ENV CARGO_BUILD_TARGET=x86_64-unknown-linux-musl

# Copy manifest files for dependency caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release --bin api-hub --target ${CARGO_BUILD_TARGET} && \
    rm -rf src

# Copy the actual source code
COPY . .

# Build the application with CPU compatibility
RUN echo "Building with RUSTFLAGS: $RUSTFLAGS" && \
    cargo build --release --bin api-hub --target ${CARGO_BUILD_TARGET}

# Runtime stage
FROM scratch

WORKDIR /

# Copy CA bundle for outbound TLS
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/api-hub /api-hub

# Expose the port your application listens on
EXPOSE 3000

USER 65532:65532

# Run the application
CMD ["/api-hub"]
