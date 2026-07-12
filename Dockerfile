FROM lukemathwalker/cargo-chef:latest AS chef
ARG DATABASE_URL
ENV DATABASE_URL $DATABASE_URL
ENV SQLX_OFFLINE true
WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo install sqlx-cli --features sqlite
RUN cargo build --release --bin botifactory
RUN sqlx database setup

FROM debian:bullseye-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates \
  # Clean up
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/botifactory botifactory
COPY configuration configuration
# Mount points for the release-binaries and database volumes.
RUN mkdir -p /releases /data
ENV BOTIFACTORY_APP_ENVIRONMENT production
ENTRYPOINT ["./botifactory"]

