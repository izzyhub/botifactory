FROM lukemathwalker/cargo-chef:latest as chef
ARG DATABASE_URL
ENV DATABASE_URL $DATABASE_URL
ENV SQLX_OFFLINE true
WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef as planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef as builder
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
ENV BOTIFACTORY_APP_ENVIRONMENT production
ENTRYPOINT ["./botifactory"]

