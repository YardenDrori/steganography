# Build stage
FROM rust:1.82-alpine AS builder

RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static

WORKDIR /app

COPY Cargo.toml ./
COPY migrations ./migrations
COPY src ./src

RUN cargo build --release

# Runtime stage
FROM alpine:latest

RUN apk add --no-cache libgcc

WORKDIR /app

COPY --from=builder /app/target/release/auth_service /app/auth_service
COPY --from=builder /app/migrations /app/migrations

EXPOSE 3001

CMD ["./auth_service"]
