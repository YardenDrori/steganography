# Multi-stage build for smaller final image
FROM rust:latest as builder

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./

# Copy all workspace members (needed for build)
COPY shared ./shared
COPY auth_service ./auth_service

# Build the auth service in release mode
RUN cargo build --release -p auth_service

# Runtime stage - smaller image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/auth_service /app/auth_service

# Copy migrations (if you have them)
COPY auth_service/migrations ./migrations

# Expose the port
EXPOSE 3001

# Run the binary
CMD ["/app/auth_service"]
