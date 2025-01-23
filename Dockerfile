# syntax=docker/dockerfile:1.7-labs

FROM rust:alpine AS build
WORKDIR /usr/src/prose-pod-api

RUN apk update && apk add musl-dev

ARG CARGO_PROFILE='release'

# Build the application.
COPY --exclude=crates/*/static/ . .
ARG CARGO_INSTALL_EXTRA_ARGS=''
ARG VERSION
ARG COMMIT
ARG BUILD_TIMESTAMP=''
RUN API_VERSION_DIR=./crates/rest-api/static/api-version && \
    mkdir -p "${API_VERSION_DIR:?}" && \
    echo "${VERSION:?}" > "${API_VERSION_DIR:?}"/VERSION && \
    echo "${COMMIT:-}" > "${API_VERSION_DIR:?}"/COMMIT && \
    if [ -z "${BUILD_TIMESTAMP}" ]; then BUILD_TIMESTAMP="$(date -u -Iseconds)" && BUILD_TIMESTAMP="${BUILD_TIMESTAMP//+00:00/Z}"; fi && \
    echo "${BUILD_TIMESTAMP:?}" > "${API_VERSION_DIR:?}"/BUILD_TIMESTAMP
RUN cargo install --path crates/rest-api --bin prose-pod-api --profile="${CARGO_PROFILE}" ${CARGO_INSTALL_EXTRA_ARGS}


FROM redocly/cli as api-docs

COPY crates/rest-api/static/api-docs .
RUN redocly bundle openapi.yaml -o /usr/share/prose-pod-api/static/api-docs/openapi.json --config redocly.cfg.yaml

COPY crates/rest-api/static/api-docs/redoc* /usr/share/prose-pod-api/static/api-docs


FROM alpine:latest

RUN apk update && apk add libgcc libc6-compat

WORKDIR /usr/share/prose-pod-api

COPY --from=build /usr/local/cargo/bin/prose-pod-api /usr/local/bin/prose-pod-api
COPY --from=api-docs /usr/share/prose-pod-api/static /usr/share/prose-pod-api/static

CMD ["prose-pod-api"]

EXPOSE 8080
