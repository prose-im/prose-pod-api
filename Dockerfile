FROM rustlang/rust:nightly-buster AS build

RUN apt-get update
RUN apt-get install -y musl-tools

RUN rustc --version && \
  rustup --version && \
  cargo --version
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/prose-pod-api
COPY . .

RUN cargo install --path . --target x86_64-unknown-linux-musl

FROM alpine:latest

COPY --from=build /usr/local/cargo/bin/prose-pod-api /usr/local/bin/prose-pod-api

CMD ["prose-pod-api"]

EXPOSE 8080
