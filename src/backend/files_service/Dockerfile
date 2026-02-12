# Build stage
FROM rust:1.91-alpine AS builder

RUN apk add --no-cache \
  musl-dev \
  pkgconf \
  openssl-dev \
  openssl-libs-static

WORKDIR /build

COPY shared/global ./shared/global
COPY files_service ./files_service

WORKDIR /build/files_service

# Use offline mode with the .sqlx cache
ENV SQLX_OFFLINE=true

# Build the release binary
RUN cargo build --release


# Runtime stage
FROM alpine:3.22

RUN apk add --no-cache libgcc

WORKDIR /app

COPY --from=builder /build/files_service/target/release/files_service /app/files_service
COPY --from=builder /build/files_service/migrations /app/migrations

EXPOSE 3004

CMD ["./files_service"]
