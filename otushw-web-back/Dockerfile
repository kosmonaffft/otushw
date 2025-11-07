# Упрощенная версия с использованием scratch образа
FROM rust:1.91.0-alpine AS builder

RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static libc-dev gcc

WORKDIR /app
COPY fake_main.rs src/main.rs
COPY Cargo.toml Cargo.toml
RUN cargo build --release
RUN rm src/main.rs

COPY migrations migrations
COPY src src
RUN cargo build --release

FROM alpine:3.22.2

COPY --from=builder /app/target/release/otushw-web-back /

ENTRYPOINT exec "/otushw-web-back"
CMD []
