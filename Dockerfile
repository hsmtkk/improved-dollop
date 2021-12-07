FROM rust:1.57 AS builder

WORKDIR /opt

COPY . .

RUN cargo build --release

FROM gcr.io/distroless/cc-debian11 AS runtime

COPY --from=builder /opt/target/release/improved-dollop /usr/local/bin/improved-dollop

ENTRYPOINT ["/usr/local/bin/improved-dollop"]
