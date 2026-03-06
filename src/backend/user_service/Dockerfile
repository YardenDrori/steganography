FROM lukemathwalker/cargo-chef:latest-rust-1-alpine AS chef
WORKDIR /build
RUN apk add --no-cache musl-dev pkgconf openssl-dev openssl-libs-static

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS cooker
COPY --from=planner /build/recipe.json recipe.json
ENV SQLX_OFFLINE=true
RUN cargo chef cook --release --recipe-path recipe.json -p user_service

FROM chef AS builder
COPY --from=cooker /build /build
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release -p user_service

FROM alpine:3.22
RUN apk add --no-cache libgcc
WORKDIR /app
COPY --from=builder /build/target/release/user_service /app/user_service
COPY --from=builder /build/user_service/migrations /app/migrations
EXPOSE 3002
CMD ["./user_service"]
