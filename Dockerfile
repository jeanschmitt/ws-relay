FROM rust:1.65 as builder

# Cache crates

RUN USER=root cargo new --bin ws-relay
WORKDIR ./ws-relay
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

# Build the actual project

COPY . .

RUN rm -f ./target/release/deps/ws_relay*
RUN cargo build --release

FROM debian:buster-slim

COPY --from=builder /ws-relay/target/release/ws-relay ./ws-relay

CMD ["/ws-relay"]
