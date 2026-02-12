# Build stage
FROM rust:1.91-alpine AS builder

RUN apk add --no-cache \
    musl-dev \
    pkgconf \
    openssl-dev \
    openssl-libs-static

WORKDIR /app

COPY Cargo.toml ./
COPY src ./src

RUN cargo build --release

# Runtime stage
FROM alpine:3.22

RUN apk add --no-cache libgcc

WORKDIR /app

COPY --from=builder /app/target/release/api_gateway /app/api_gateway

EXPOSE 3000

CMD ["./api_gateway"]
