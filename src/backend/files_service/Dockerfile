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
    libavdevice-dev \
    && rm -rf /var/lib/apt/lists/*
ENV PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/share/pkgconfig

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS cooker
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json -p files_service

FROM chef AS builder
COPY --from=cooker /build /build
COPY . .
RUN cargo build --release -p files_service

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ffmpeg \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /build/target/release/files_service /app/files_service
EXPOSE 3004
CMD ["./files_service"]
