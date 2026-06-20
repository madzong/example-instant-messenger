# ------
# RECIPE
# ------
ARG RUST_VERSION=1.95.0
ARG APP_NAME=rest-backend
FROM lukemathwalker/cargo-chef:latest-rust-${RUST_VERSION}-slim AS planner
ARG APP_NAME
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY --parents **/Cargo.toml ./
# Create dummy files for every member
RUN <<EOF
sed -n '/members = \[/,/\]/ { /"/ s/^[[:space:]]*"\([^"]*\)",.*/\1/p }' Cargo.toml | while read -r member; do
    mkdir -p "$member/src"
    if [ member = "instant-messenger-common" ]; then
        echo "" > "$member/src/lib.rs"
    else
        echo "fn main() {}" > "$member/src/main.rs"
    fi
done
EOF
RUN cargo chef prepare --recipe-path recipe.json

# ------------
# DEPENDENCIES
# ------------
FROM lukemathwalker/cargo-chef:latest-rust-${RUST_VERSION}-slim AS builder
ARG APP_NAME
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# ---
# APP
# ---
COPY instant-messenger-common/ instant-messenger-common/
COPY ${APP_NAME}/ ${APP_NAME}/
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release --bin ${APP_NAME} -p ${APP_NAME}

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
