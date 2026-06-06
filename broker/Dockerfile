ARG RUST_VERSION=1.95.0
ARG APP_NAME=broker
FROM rust:${RUST_VERSION}-slim-bullseye AS build
ARG APP_NAME
WORKDIR /app

RUN --mount=type=bind,source=${APP_NAME},target=${APP_NAME}/,rw \
    --mount=type=bind,source=instant_messenger_common/,target=instant_messenger_common/,rw \
    --mount=type=cache,target=/app/${APP_NAME}/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    <<EOF
set -e
cd $APP_NAME
cargo build --release
cp ./target/release/$APP_NAME /bin/server
EOF

FROM debian:bullseye-slim AS final

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

COPY --from=build /bin/server /bin/

CMD ["/bin/server"]
