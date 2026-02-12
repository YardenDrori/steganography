# Build stage
FROM rust:1.91-alpine AS builder

  RUN apk add --no-cache \
    musl-dev \
    pkgconf \
    openssl-dev \
    openssl-libs-static

WORKDIR /build

COPY shared/global ./shared/global
COPY steganography_service ./steganography_service 

WORKDIR /build/steganography_service 

# Build the release binary
RUN cargo build --release


# Runtime stage
FROM alpine:3.22

RUN apk add --no-cache libgcc

WORKDIR /app

COPY --from=builder /build/steganography_service/target/release/steganography_service /app/steganography_service

EXPOSE 3003

CMD ["./steganography_service"]
