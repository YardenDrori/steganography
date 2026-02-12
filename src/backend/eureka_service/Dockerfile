# Build stage
FROM rust:1.91-alpine AS builder

RUN apk add --no-cache \
  musl-dev \
  pkgconf \
  openssl-dev \
  openssl-libs-static

WORKDIR /build

COPY shared/global ./shared/global
COPY eureka_service ./eureka_service

WORKDIR /build/eureka_service

ENV SQLX_OFFLINE=true

RUN cargo build --release


# Runtime stage
FROM alpine:3.22

RUN apk add --no-cache libgcc

WORKDIR /app

COPY --from=builder /build/eureka_service/target/release/eureka_service /app/eureka_service
COPY --from=builder /build/eureka_service/migrations /app/migrations

EXPOSE 3005

CMD ["./eureka_service"]
