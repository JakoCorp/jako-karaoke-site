# Chef
FROM rust:1.96.0-trixie AS chef
RUN cargo install cargo-chef --version 0.1.77 --locked
WORKDIR /app

# Planner
FROM chef AS planner

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo chef prepare --recipe-path recipe.json

# Builder
FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo chef cook --release --recipe-path recipe.json
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo build --release --bin server --bin backup

# Runtime
FROM debian:trixie-20260623-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    gosu \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --no-create-home --uid 1000 appuser

WORKDIR /app

COPY --from=builder /app/target/release/server /app/target/release/backup /usr/local/bin/
COPY docker-entrypoint.sh /usr/local/bin/

RUN chmod +x /usr/local/bin/docker-entrypoint.sh

ARG GIT_COMMIT=unknown

EXPOSE 3000
ENV RUST_LOG=info
ENV GIT_COMMIT=$GIT_COMMIT

ENTRYPOINT ["docker-entrypoint.sh"]
CMD ["server"]
