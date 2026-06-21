ARG RUST_VERSION=1.95.0
ARG APP_NAME=broker
FROM rust:${RUST_VERSION}-slim-trixie AS builder
ARG APP_NAME
WORKDIR /app
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/${APP_NAME}/target \
    cargo build --release --bin ${APP_NAME} -p ${APP_NAME}

FROM debian:trixie-slim AS final
ARG APP_NAME
WORKDIR /app

RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

ARG UID=10001
RUN useradd \
    --system \
    --shell /usr/sbin/nologin \
    --no-create-home \
    --home-dir /nonexistent \
    --uid "${UID}" \
    appuser
USER appuser

COPY --from=builder /app/target/release/${APP_NAME} /app/server

CMD ["./server"]
