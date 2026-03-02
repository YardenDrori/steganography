FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /build
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    clang \
    libavcodec-dev \
    libavformat-dev \
    libavutil-dev \
    libswscale-dev \
    libavfilter-dev \
    libswresample-dev \
    && rm -rf /var/lib/apt/lists/*

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS cooker
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM chef AS builder
COPY --from=cooker /build /build
COPY . .
RUN cargo build --release -p steganography_service

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ffmpeg \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /build/target/release/steganography_service /app/steganography_service
EXPOSE 3003
CMD ["./steganography_service"]
