FROM alpine:latest

RUN apk update && apk add libgcc libc6-compat

ARG RUST_OUT_DIR

COPY ${RUST_OUT_DIR:?}/prose-pod-api /usr/local/bin/prose-pod-api
COPY ./crates/api/static /static

CMD ["prose-pod-api"]

EXPOSE 8080
