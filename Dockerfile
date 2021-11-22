FROM rust:1.51.0 as build

RUN rustup default nightly-2021-06-17
RUN apt-get update && apt-get install -y clang

WORKDIR /build
COPY . /build
RUN make release

FROM debian:buster

COPY --from=build /build/target/release/setheum /usr/local/bin
ENTRYPOINT ["setheum-node"]
