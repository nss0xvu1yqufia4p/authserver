ARG BASE_IMAGE=ekidd/rust-musl-builder:1.57.0

FROM ${BASE_IMAGE} AS builder

ADD --chown=rust:rust . ./

RUN cargo build --release
RUN strip /home/rust/src/target/x86_64-unknown-linux-musl/release/authserver

FROM alpine:3.15.0
RUN apk --no-cache add ca-certificates
COPY --from=builder \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/authserver \
    /usr/local/bin/
CMD /usr/local/bin/authserver
