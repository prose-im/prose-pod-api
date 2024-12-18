# syntax=docker/dockerfile:1.7-labs

FROM lukemathwalker/cargo-chef:latest-rust-alpine AS chef
WORKDIR /usr/src/prose-pod-api


FROM chef AS planner
COPY --exclude=crates/*/static/ . .
RUN cargo chef prepare --recipe-path recipe.json


FROM chef AS builder
COPY --from=planner /usr/src/prose-pod-api/recipe.json recipe.json
RUN apk update && apk add musl-dev

ARG CARGO_PROFILE='release'

# Build the dependencies (and use Docker’s cache to avoid rebuilding it next time).
ARG CARGO_CHEF_EXTRA_ARGS=''
RUN cargo chef cook --profile="${CARGO_PROFILE}" --recipe-path recipe.json ${CARGO_CHEF_EXTRA_ARGS}

# Build the application.
COPY --exclude=crates/*/static/ . .
ARG CARGO_INSTALL_EXTRA_ARGS=''
RUN cargo install --path crates/rest-api --bin prose-pod-api --profile="${CARGO_PROFILE}" ${CARGO_INSTALL_EXTRA_ARGS}


FROM redocly/cli as api-docs

COPY crates/rest-api/static/api-docs .
RUN redocly bundle openapi.yaml -o /usr/share/prose-pod-api/static/api-docs/openapi.json

COPY crates/rest-api/static/api-docs/redoc* /usr/share/prose-pod-api/static/api-docs


FROM alpine:latest

RUN apk update && apk add libgcc libc6-compat

WORKDIR /usr/share/prose-pod-api

COPY --from=builder /usr/local/cargo/bin/prose-pod-api /usr/local/bin/prose-pod-api
COPY --from=api-docs /usr/share/prose-pod-api/static /usr/share/prose-pod-api/static

CMD ["prose-pod-api"]

EXPOSE 8080
