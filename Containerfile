FROM docker.io/rust:1.76-bullseye as builder
WORKDIR /build
COPY . .

RUN mkdir /data &&\
	cargo build --release

FROM gcr.io/distroless/cc-debian11
COPY --from=builder /build/target/release/abby_bot /bot
COPY --from=builder /data /data
VOLUME [ "/data" ]

ENTRYPOINT [ "/bot" ]
