FROM rust:1-bookworm AS build
WORKDIR /app
COPY . .
RUN cargo build --release -p liquidity-hub-app

FROM debian:bookworm-slim
RUN useradd --create-home --shell /usr/sbin/nologin app
WORKDIR /app
COPY --from=build /app/target/release/liquidity-hub-app /usr/local/bin/liquidity-hub-app
EXPOSE 8080
USER app
ENTRYPOINT ["/usr/local/bin/liquidity-hub-app"]
