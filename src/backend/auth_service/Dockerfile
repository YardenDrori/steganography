# Build stage
FROM rust:1.91-alpine AS builder

RUN apk add --no-cache \
  musl-dev \
  pkgconf \
  openssl-dev \
  openssl-libs-static

WORKDIR /build

COPY shared/global ./shared/global
COPY shared/auth_user ./shared/auth_user
COPY auth_service ./auth_service

WORKDIR /build/auth_service

# Use offline mode with the .sqlx cache
ENV SQLX_OFFLINE=true

# Build the release binary
RUN cargo build --release


# Runtime stage
FROM alpine:3.22

RUN apk add --no-cache libgcc

WORKDIR /app

COPY --from=builder /build/auth_service/target/release/auth_service /app/auth_service
COPY --from=builder /build/auth_service/migrations /app/migrations

EXPOSE 3001

CMD ["./auth_service"]
