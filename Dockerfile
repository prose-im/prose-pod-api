FROM rust:alpine AS build

ARG CARGO_INSTALL_EXTRA_ARGS=''

RUN apk update && apk add musl-dev

RUN rustc --version && \
    rustup --version && \
    cargo --version

WORKDIR /usr/src/prose-pod-api
COPY . .

RUN cargo install --path crates/rest-api ${CARGO_INSTALL_EXTRA_ARGS}

FROM alpine:latest

RUN apk update && apk add libgcc libc6-compat

COPY --from=build /usr/local/cargo/bin/prose-pod-api /usr/local/bin/prose-pod-api
COPY --from=build /usr/src/prose-pod-api/crates/rest-api/static /static

CMD ["prose-pod-api"]

EXPOSE 8080
