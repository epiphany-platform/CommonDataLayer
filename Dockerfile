# syntax=docker/dockerfile:experimental

# -----------------
# Cargo Build Stage
# -----------------

FROM clux/muslrust:1.46.0-stable as cargo-build

# For librdkafka
RUN apt-get update && apt-get install -y cmake

WORKDIR /usr/src/cdl/
COPY rust-toolchain ./
COPY Cargo.lock ./
COPY Cargo.toml ./

COPY crates/blob-store/Cargo.toml crates/blob-store/Cargo.toml
COPY crates/benchmarking/Cargo.toml crates/benchmarking/Cargo.toml
COPY crates/cdl-cli/Cargo.toml crates/cdl-cli/Cargo.toml
COPY crates/command-service/Cargo.toml crates/command-service/Cargo.toml
COPY crates/data-router/Cargo.toml crates/data-router/Cargo.toml
COPY crates/db-shrinker-postgres/Cargo.toml crates/db-shrinker-postgres/Cargo.toml
COPY crates/document-storage/Cargo.toml crates/document-storage/Cargo.toml
COPY crates/leader-elector/Cargo.toml crates/leader-elector/Cargo.toml
COPY crates/query-router/Cargo.toml crates/query-router/Cargo.toml
COPY crates/query-service/Cargo.toml crates/query-service/Cargo.toml
COPY crates/query-service-ts/Cargo.toml crates/query-service-ts/Cargo.toml
COPY crates/rpc/Cargo.toml crates/rpc/Cargo.toml
COPY crates/schema-registry/Cargo.toml crates/schema-registry/Cargo.toml
COPY crates/utils/Cargo.toml crates/utils/Cargo.toml
COPY crates/api/Cargo.toml crates/api/Cargo.toml

RUN cargo fetch
RUN rustup target add x86_64-unknown-linux-musl

COPY crates/blob-store/ crates/blob-store/
COPY crates/benchmarking/ crates/benchmarking/
COPY crates/cdl-cli/ crates/cdl-cli/
COPY crates/command-service/ crates/command-service/
COPY crates/data-router/ crates/data-router/
COPY crates/db-shrinker-postgres/ crates/db-shrinker-postgres/
COPY crates/document-storage/ crates/document-storage/
COPY crates/leader-elector/ crates/leader-elector/
COPY crates/query-router/ crates/query-router/
COPY crates/query-service/ crates/query-service/
COPY crates/query-service-ts/ crates/query-service-ts/
COPY crates/rpc/ crates/rpc/
COPY crates/schema-registry/ crates/schema-registry/
COPY crates/utils/ crates/utils/
COPY crates/api/ crates/api/

ARG ENV
ARG BIN

RUN --mount=type=cache,mode=0755,target=/usr/local/cargo/registry \
    --mount=type=cache,mode=0755,target=/usr/src/cdl/target \
    if [ "$ENV" = "DEV" ]; \
    then CARGO_ARGS="--offline"; CARGO_PROFILE="debug"; \
    else CARGO_ARGS="--offline --release"; CARGO_PROFILE="release"; \
    fi && \
    LIB_LDFLAGS=-L/usr/lib/x86_64-linux-gnu CFLAGS=-I/usr/local/musl/include CC=musl-gcc CXX=g++ \
    cargo build $CARGO_ARGS --workspace $FEATURE_FLAGS && \
    mkdir output && \
    bash -c "cp target/x86_64-unknown-linux-musl/$CARGO_PROFILE/$BIN output/"

RUN if [ "$ENV" != "DEV" ]; \
    then for f in output/*; do strip $f; done; fi

# -----------------
# Final Stage
# -----------------

FROM alpine

COPY --from=cargo-build /usr/src/cdl/output/* /bin/
COPY crates/benchmarking/sample_json sample_json/
