# Build stage
FROM rust:1.90-alpine AS builder

RUN apk add --no-cache \
    musl-dev=1.2.5-r10 \
    pkgconf=2.4.3-r0 \
    openssl-dev=3.5.4-r0 \
    openssl-libs-static=3.5.4-r0

WORKDIR /app

COPY Cargo.toml ./
COPY src ./src

RUN cargo build --release

# Runtime stage
FROM alpine:3.22

RUN apk add --no-cache libgcc=14.2.0-r6

WORKDIR /app

COPY --from=builder /app/target/release/api_gateway /app/api_gateway

EXPOSE 3000

CMD ["./api_gateway"]
