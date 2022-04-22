# Workaround for QEmu bug when building for 32bit platforms on a 64bit host
FROM --platform=$BUILDPLATFORM rust:bullseye as vendor
ARG BUILDPLATFORM
ARG TARGETPLATFORM
RUN echo "Running on: $BUILDPLATFORM / Building for $TARGETPLATFORM"
WORKDIR /app

COPY ./Cargo.toml .
COPY ./Cargo.lock .
COPY ./src src
RUN mkdir .cargo && cargo vendor > .cargo/config.toml

FROM rust:bullseye as builder
WORKDIR /app

COPY --from=vendor /app/.cargo .cargo
COPY --from=vendor /app/vendor vendor
COPY ./Cargo.toml .
COPY ./Cargo.lock .
COPY ./src src
RUN rustup toolchain install nightly-x86_64-unknown-linux-gnu && rustup toolchain install nightly-armv7-unknown-linux-gnueabihf && rustup toolchain install nightly-aarch64-unknown-linux-gnu
RUN cargo +nightly build --release

FROM debian:bullseye-slim
WORKDIR /app
ENV RUST_BACKTRACE=full
COPY --from=builder /app/target/release/gexport gexport
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && update-ca-certificates

ENTRYPOINT ["/app/gexport"]