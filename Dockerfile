FROM rust:1.70-slim-buster AS dev

WORKDIR /app

RUN cargo install cargo-watch

EXPOSE 8080


FROM rust:1.70-slim-buster AS final

WORKDIR /app

COPY . .

EXPOSE 8080

RUN cargo build --release

RUN mv ./target/release/metered_api_server .

RUN cargo clean

CMD ["./metered_api_server"]
