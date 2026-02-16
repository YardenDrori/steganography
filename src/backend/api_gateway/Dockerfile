# Build stage
  FROM rust:1.91-alpine AS builder

  RUN apk add --no-cache \
      musl-dev \
      pkgconf \
      openssl-dev \
      openssl-libs-static

  WORKDIR /build

  # Copy shared dependencies
  COPY shared/global ./shared/global
  COPY shared/auth_user ./shared/auth_user

  # Copy the api_gateway service
  COPY api_gateway ./api_gateway

  WORKDIR /build/api_gateway

  # Build the release binary
  RUN cargo build --release

  # Runtime stage
  FROM alpine:3.22

  RUN apk add --no-cache libgcc

  WORKDIR /app

  COPY --from=builder /build/api_gateway/target/release/api_gateway /app/api_gateway

  EXPOSE 3000

  CMD ["./api_gateway"]
