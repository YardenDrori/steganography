# Build stage
FROM rust:1.90-alpine AS builder

RUN apk add --no-cache \
  musl-dev=1.2.5-r10 \
  pkgconf=2.4.3-r0 \
  openssl-dev=3.5.4-r0 \
  openssl-libs-static=3.5.4-r0

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

RUN apk add --no-cache libgcc=14.2.0-r6

WORKDIR /app

COPY --from=builder /build/auth_service/target/release/auth_service /app/auth_service
COPY --from=builder /build/auth_service/migrations /app/migrations

EXPOSE 3001

CMD ["./auth_service"]
