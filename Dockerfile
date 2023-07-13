FROM rust:1.70-slim-buster

WORKDIR /app

COPY . .

EXPOSE 8080

RUN cargo build --release

RUN mv ./target/release/metered_api_server .

RUN cargo clean

CMD ["./metered_api_server"]
