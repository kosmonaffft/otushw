# Упрощенная версия с использованием scratch образа
FROM rust:1.94.0-trixie AS build

WORKDIR /app
COPY Cargo.toml Cargo.toml
COPY pg_migrations pg_migrations
COPY citus_migrations citus_migrations
COPY src src
RUN cargo build --release

FROM debian:trixie

COPY --from=build /app/target/release/otushw-web-back /otushw-web-back

EXPOSE 8080

CMD ["/otushw-web-back"]
