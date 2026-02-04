# Упрощенная версия с использованием scratch образа
FROM rust:1.93-trixie AS build

WORKDIR /app
COPY Cargo.toml Cargo.toml
COPY migrations migrations
COPY src src
RUN cargo build --release

FROM debian:trixie

COPY --from=build /app/target/release/otushw-web-back /otushw-web-back

EXPOSE 8080

CMD ["/otushw-web-back"]
