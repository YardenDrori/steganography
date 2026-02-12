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
COPY user_service ./user_service

WORKDIR /build/user_service

# Use offline mode with the .sqlx cache
ENV SQLX_OFFLINE=true

RUN cargo build --release

# Runtime stage
FROM alpine:3.22

RUN apk add --no-cache libgcc

WORKDIR /app

COPY --from=builder /build/user_service/target/release/user_service /app/user_service
COPY --from=builder /build/user_service/migrations /app/migrations

EXPOSE 3002

CMD ["./user_service"]
