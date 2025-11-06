# Build stage
FROM rust:1.82-alpine AS builder

RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    openssl-libs-static \
    ffmpeg-dev \
    clang-dev \
    llvm-dev

WORKDIR /app

COPY Cargo.toml ./
COPY src ./src

RUN cargo build --release

# Runtime stage
FROM alpine:latest

RUN apk add --no-cache libgcc ffmpeg

WORKDIR /app

RUN mkdir -p /app/files

COPY --from=builder /app/target/release/steganography_service /app/steganography_service

EXPOSE 3002

CMD ["./steganography_service"]
