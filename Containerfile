FROM docker.io/rust:1.86-bookworm as builder
WORKDIR /build
COPY . .

RUN mkdir /data &&\
	cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /build/target/release/abby_bot /bot
COPY --from=builder /data /data
VOLUME [ "/data" ]

ENTRYPOINT [ "/bot" ]
