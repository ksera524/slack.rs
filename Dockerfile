FROM nixos/nix:2.24.11 AS builder

WORKDIR /src

COPY . .

# Build static binary and collect CA bundle via Nix
RUN nix build .#api-hub-musl \
      --extra-experimental-features 'nix-command flakes' \
      --no-link \
      --no-write-lock-file \
      --print-out-paths > /tmp/api-hub.out && \
    cp "$(cat /tmp/api-hub.out)/bin/api-hub" /tmp/api-hub && \
    chmod +x /tmp/api-hub && \
    nix build .#ca-certificates \
      --extra-experimental-features 'nix-command flakes' \
      --no-link \
      --no-write-lock-file \
      --print-out-paths > /tmp/cacert.out && \
    cp "$(cat /tmp/cacert.out)/etc/ssl/certs/ca-bundle.crt" /tmp/ca-certificates.crt

# Runtime stage
FROM scratch

WORKDIR /

# Copy CA bundle for outbound TLS
COPY --from=builder /tmp/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

# Copy the binary from the builder stage
COPY --from=builder /tmp/api-hub /api-hub

# Expose the port your application listens on
EXPOSE 3000

USER 65532:65532

# Run the application
CMD ["/api-hub"]
