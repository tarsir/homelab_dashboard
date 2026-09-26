FROM rust:1-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY src ./src

RUN touch src/main.rs && cargo build --release

# Runtime stage

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
  docker.io \
  ca-certificates \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/homelab_dashboard /app/homelab_dashboard
COPY static ./static

ENV PORT=7001
EXPOSE 7001

ENTRYPOINT ["/app/homelab_dashboard"]
