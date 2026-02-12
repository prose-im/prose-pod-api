FROM lukemathwalker/cargo-chef:0.1.72-rust-1.88-alpine3.22 AS chef
WORKDIR /app


FROM chef AS plan
COPY . .
RUN cargo chef prepare --recipe-path recipe.json


FROM chef AS build

RUN apk update && apk add musl-dev

COPY --from=plan /app/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --recipe-path recipe.json

ARG CARGO_PROFILE='release'

# Build application
COPY . .
ARG CARGO_INSTALL_EXTRA_ARGS=''
ARG VERSION
ARG COMMIT
ARG BUILD_TIMESTAMP=''
RUN API_VERSION_DIR=./src/service/static/api-version && \
    mkdir -p "${API_VERSION_DIR:?}" && \
    echo "${VERSION:?}" > "${API_VERSION_DIR:?}"/VERSION && \
    echo "${COMMIT:-}" > "${API_VERSION_DIR:?}"/COMMIT && \
    if [ -z "${BUILD_TIMESTAMP}" ]; then BUILD_TIMESTAMP="$(date -u -Iseconds)" && BUILD_TIMESTAMP="${BUILD_TIMESTAMP//+00:00/Z}"; fi && \
    echo "${BUILD_TIMESTAMP:?}" > "${API_VERSION_DIR:?}"/BUILD_TIMESTAMP
RUN cargo install --path src/rest-api --bin prose-pod-api --profile="${CARGO_PROFILE}" -F openapi ${CARGO_INSTALL_EXTRA_ARGS}


FROM redocly/cli:1.34.5 AS api-docs

COPY docs/openapi .
RUN redocly bundle openapi.yaml -o /usr/share/prose-pod-api/static/api-docs/openapi.json --config redocly.cfg.yaml


FROM alpine:3.22.0 AS run

RUN apk update && apk add libgcc libc6-compat

WORKDIR /usr/share/prose-pod-api

COPY --from=build /usr/local/cargo/bin/prose-pod-api /usr/local/bin/prose-pod-api
COPY --from=api-docs /usr/share/prose-pod-api/static /usr/share/prose-pod-api/static
COPY prose.lic /usr/share/prose/prose.lic

VOLUME /etc/prose/
VOLUME /var/lib/prose-pod-api/

CMD ["prose-pod-api"]

EXPOSE 8080
