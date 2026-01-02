# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.84.0
ARG APP_NAME=app

FROM rust:${RUST_VERSION}-slim-bookworm AS build
ARG APP_NAME
ARG DATABASE_URL
WORKDIR /app

ENV DATABASE_URL=${DATABASE_URL}

RUN --mount=type=bind,source=src,target=src \
  --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
  --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
  --mount=type=cache,target=/app/target/ \
  --mount=type=cache,target=/usr/local/cargo/git/db \
  --mount=type=cache,target=/usr/local/cargo/registry/ \
  set -eux; \
  cargo build --release --locked; \
  cp ./target/release/$APP_NAME /bin/server

FROM debian:bookworm-slim AS final

RUN adduser \
  --disabled-password \
  --gecos "" \
  --home "/nonexistent" \
  --shell "/sbin/nologin" \
  --no-create-home \
  --uid "1001" \
  appuser
USER appuser

COPY --from=build /bin/server /bin/

EXPOSE 8080

CMD ["/bin/server"]
