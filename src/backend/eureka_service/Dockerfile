# Build stage
FROM rust:1.90-alpine AS builder

RUN apk add --no-cache \
  musl-dev=1.2.5-r10 \
  pkgconf=2.4.3-r0 \
  openssl-dev=3.5.4-r0 \
  openssl-libs-static=3.5.4-r0

WORKDIR /build

COPY shared/global ./shared/global
COPY eureka_service ./eureka_service

WORKDIR /build/eureka_service

ENV SQLX_OFFLINE=true

RUN cargo build --release


# Runtime stage
FROM alpine:3.22

RUN apk add --no-cache libgcc=14.2.0-r6

WORKDIR /app

COPY --from=builder /build/eureka_service/target/release/eureka_service /app/eureka_service
COPY --from=builder /build/eureka_service/migrations /app/migrations

EXPOSE 3005

CMD ["./eureka_service"]
