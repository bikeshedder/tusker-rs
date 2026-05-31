# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.96

FROM rust:${RUST_VERSION}-alpine AS builder
WORKDIR /src

RUN rustup target add x86_64-unknown-linux-musl

COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY examples/ examples/

RUN cargo build --locked --release -p tusker --target x86_64-unknown-linux-musl

FROM scratch

COPY --from=builder /src/target/x86_64-unknown-linux-musl/release/tusker /tusker

ENTRYPOINT ["/tusker"]
