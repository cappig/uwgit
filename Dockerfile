FROM rust:1-slim-bookworm AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock build.rs ./
COPY src src
COPY templates templates
COPY static static
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates git libssl3 && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/uwgit /app/uwgit
COPY --from=builder /app/static /app/static

EXPOSE 3000
CMD ["/app/uwgit"]
