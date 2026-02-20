FROM lukemathwalker/cargo-chef:latest-rust-1-alpine AS chef
WORKDIR /build
RUN apk add --no-cache musl-dev pkgconf openssl-dev openssl-libs-static

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

FROM alpine:3.22
RUN apk add --no-cache libgcc
WORKDIR /app
COPY --from=builder /build/target/release/steganography_service /app/steganography_service
EXPOSE 3003
CMD ["./steganography_service"]
