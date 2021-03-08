FROM ekidd/rust-musl-builder:stable AS builder

COPY --chown=rust:rust . ./

RUN cargo build --bin skysold --release

# -------------------

FROM alpine:latest

RUN apk --no-cache add ca-certificates

COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/skysold /app/skysold

ENTRYPOINT ["/app/skysold"]
