# Упрощенная версия с использованием scratch образа
FROM rust:1.91.0-slim-bullseye

RUN apt-get update && apt-get install -y pkg-config libssl-dev

WORKDIR /app
COPY fake_main.rs src/main.rs
COPY Cargo.toml Cargo.toml
RUN cargo build
RUN rm src/main.rs

COPY migrations migrations
COPY src src
RUN cargo build

EXPOSE 8080

CMD ["/app/target/debug/otushw-web-back"]
