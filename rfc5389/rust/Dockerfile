FROM rust:alpine AS builder

WORKDIR /usr/src/app
COPY . .

RUN cargo install --path .

FROM alpine

COPY --from=builder /usr/local/cargo/bin/rfc5389 /bin/rfc5389

CMD ["/bin/rfc5389"]
