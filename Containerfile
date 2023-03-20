FROM docker.io/rust:1.68-bullseye as builder
WORKDIR /build
COPY . .

RUN cargo build --release

FROM docker.io/debian:bullseye-slim
RUN mkdir -p /abby_bot/data
COPY --from=builder /build/target/release/abby_bot /abby_bot/bot
WORKDIR /abby_bot

ENTRYPOINT [ "/abby_bot/bot" ]
