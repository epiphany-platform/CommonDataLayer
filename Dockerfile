# syntax=docker/dockerfile:experimental

# -----------------
# CI related
# -----------------
FROM alpine as cache-export
RUN --mount=type=cache,mode=0755,target=/usr/src/cdl/target \
    --mount=type=cache,mode=0755,target=/root/.cargo/registry \
    tar cfz cache_target.tar.gz -C /usr/src/cdl/target . && \
    tar cfz cache_registry.tar.gz -C /root/.cargo/registry .

FROM local/cache as cache-import
RUN --mount=type=cache,mode=0755,target=/usr/src/cdl/target \
    --mount=type=cache,mode=0755,target=/root/.cargo/registry \
    tar xfz cache_target.tar.gz -C /usr/src/cdl/target && \
    tar xfz cache_registry.tar.gz -C /root/.cargo/registry

# -----------------
# Cargo Build Stage
# -----------------

FROM clux/muslrust:1.46.0-stable as cargo-build

# For librdkafka
RUN apt-get update && apt-get install -y cmake

WORKDIR /usr/src/cdl/
COPY rust-toolchain ./
RUN rustup target add x86_64-unknown-linux-musl
COPY Cargo.lock Cargo.toml ./

COPY benchmarking/ benchmarking/
COPY crates/ crates/

ARG ENV

RUN --mount=type=cache,mode=0755,target=/root/.cargo/registry \
    --mount=type=cache,mode=0755,target=/root/.cargo/git \
    --mount=type=cache,mode=0755,target=/usr/src/cdl/target \
    if [ "$ENV" = "DEV" ]; \
    then CARGO_ARGS="--out-dir=output -Z unstable-options"; \
    elif [ "$ENV" = "CI" ]; \
    then CARGO_ARGS="--out-dir=output -Z unstable-options --profile ci"; \
    else CARGO_ARGS="--out-dir=output -Z unstable-options --release"; \
    fi && \
    LIB_LDFLAGS=-L/usr/lib/x86_64-linux-gnu CFLAGS=-I/usr/local/musl/include CC=musl-gcc CXX=g++ \
    cargo build $CARGO_ARGS --workspace

RUN if [ "$ENV" != "DEV" ]; \
    then for f in output/*; do strip $f; done; fi

# -----------------
# Final Stage
# -----------------

FROM alpine

ARG BIN
COPY --from=cargo-build /usr/src/cdl/output/$BIN /bin/
