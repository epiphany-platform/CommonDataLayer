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

COPY blob-store/Cargo.toml blob-store/Cargo.toml
COPY benchmarking/Cargo.toml benchmarking/Cargo.toml
COPY cdl-cli/Cargo.toml cdl-cli/Cargo.toml
COPY command-service/Cargo.toml command-service/Cargo.toml
COPY data-router/Cargo.toml data-router/Cargo.toml
COPY db-shrinker-postgres/Cargo.toml db-shrinker-postgres/Cargo.toml
COPY document-storage/Cargo.toml document-storage/Cargo.toml
COPY leader-elector/Cargo.toml leader-elector/Cargo.toml
COPY query-router/Cargo.toml query-router/Cargo.toml
COPY query-service/Cargo.toml query-service/Cargo.toml
COPY query-service-ts/Cargo.toml query-service-ts/Cargo.toml
COPY schema-registry/Cargo.toml schema-registry/Cargo.toml
COPY utils/Cargo.toml utils/Cargo.toml
COPY rpc/Cargo.toml rpc/Cargo.toml

RUN cargo fetch
RUN rustup target add x86_64-unknown-linux-musl

COPY blob-store/ blob-store/
COPY benchmarking/ benchmarking/
COPY cdl-cli/ cdl-cli/
COPY command-service/ command-service/
COPY data-router/ data-router/
COPY db-shrinker-postgres/ db-shrinker-postgres/
COPY document-storage/ document-storage/
COPY leader-elector/ leader-elector/
COPY query-router/ query-router/
COPY query-service/ query-service/
COPY query-service-ts/ query-service-ts/
COPY schema-registry/ schema-registry/
COPY utils/ utils/
COPY rpc/ rpc/

ARG ENV
ARG BIN

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/cdl/target \
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
COPY benchmarking/sample_json sample_json/
