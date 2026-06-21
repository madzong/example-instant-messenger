ARG RUST_VERSION=1.95.0
ARG APP_NAME=frontend
FROM rust:${RUST_VERSION}-slim-trixie AS build
ARG APP_NAME
WORKDIR /app

RUN rustup target add wasm32-unknown-unknown
RUN apt-get update && \
    apt-get install -y wget pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*
RUN wget -qO- https://github.com/trunk-rs/trunk/releases/latest/download/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf - -C /usr/local/bin

COPY Cargo.toml Cargo.lock ./
COPY . .

WORKDIR /app/frontend
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/${APP_NAME}/target \
    trunk build --release

FROM nginx:alpine AS final
ARG APP_NAME
COPY --from=build /app/${APP_NAME}/dist /usr/share/nginx/html
COPY ${APP_NAME}/nginx.conf /etc/nginx/conf.d/default.conf
CMD ["nginx", "-g", "daemon off;"]
