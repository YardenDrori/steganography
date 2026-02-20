FROM lukemathwalker/cargo-chef:latest-rust-1-alpine AS chef
WORKDIR /build
RUN apk add --no-cache musl-dev pkgconf openssl-dev openssl-libs-static

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS cooker
COPY --from=planner /build/recipe.json recipe.json
ENV SQLX_OFFLINE=true
RUN cargo chef cook --release --recipe-path recipe.json

FROM chef AS builder
COPY --from=cooker /build /build
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release -p files_service

FROM alpine:3.22
RUN apk add --no-cache libgcc
WORKDIR /app
COPY --from=builder /build/target/release/files_service /app/files_service
COPY --from=builder /build/files_service/migrations /app/migrations
EXPOSE 3004
CMD ["./files_service"]
