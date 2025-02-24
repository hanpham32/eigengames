FROM rustlang/rust:nightly AS chef

RUN cargo install cargo-chef
WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN cargo chef prepare --recipe-path recipe.json
RUN cargo chef cook --recipe-path recipe.json

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=chef /app/target/release/my-eigenlayer-avs-1 /usr/local/bin

LABEL org.opencontainers.image.authors="Han <phamhan@uw.edu>"
LABEL org.opencontainers.image.description="eigengames!"
LABEL org.opencontainers.image.source="https://github.com/hanpham32/my-eigenlayer-avs-1"
LABEL org.opencontainers.image.licenses="MIT OR Apache-2.0"

ENV RUST_LOG="gadget=info"
ENV BIND_ADDR="0.0.0.0"
ENV BIND_PORT=9632
ENV BLUEPRINT_ID=0
ENV SERVICE_ID=0

ENTRYPOINT ["/usr/local/bin/my-eigenlayer-avs-1", "run"]