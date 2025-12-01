FROM rust:alpine AS build
WORKDIR /usr/src/prose-pod-api

RUN apk add --no-cache musl-dev

ARG CARGO_PROFILE='release'

# Build the application.
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
RUN cargo install --path src/rest-api --bin prose-pod-api --profile="${CARGO_PROFILE}" ${CARGO_INSTALL_EXTRA_ARGS}


FROM alpine:latest

RUN apk add --no-cache libgcc libc6-compat

WORKDIR /usr/share/prose-pod-api

COPY --from=build /usr/local/cargo/bin/prose-pod-api /usr/local/bin/prose-pod-api
COPY prose.lic /usr/share/prose/prose.lic

VOLUME /etc/prose/
VOLUME /var/lib/prose-pod-api/

CMD ["prose-pod-api"]

EXPOSE 8080
