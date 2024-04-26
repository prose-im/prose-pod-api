FROM rustlang/rust:nightly-buster AS build

ARG CARGO_INSTALL_EXTRA_ARGS=''

RUN apt-get update
RUN apt-get install -y musl-tools

RUN rustc --version && \
    rustup --version && \
    cargo --version
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/prose-pod-api
COPY . .

RUN cargo install --path . --target x86_64-unknown-linux-musl $CARGO_INSTALL_EXTRA_ARGS

FROM alpine:latest

COPY --from=build /usr/local/cargo/bin/prose-pod-api /usr/local/bin/prose-pod-api
COPY --from=build /usr/src/prose-pod-api/static /static

CMD ["prose-pod-api"]

EXPOSE 8080
