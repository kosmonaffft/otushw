# Упрощенная версия с использованием scratch образа
FROM rust:1.91.1-alpine3.22 AS build

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static

WORKDIR /app
COPY Cargo.toml Cargo.toml
COPY migrations migrations
COPY src src
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch

COPY --from=build /app/target/x86_64-unknown-linux-musl/release/otushw-web-back /otushw-web-back

EXPOSE 8080

CMD ["/otushw-web-back"]
