FROM rust:1.58 AS build_cache

# Install rust components
RUN rustup component add rustfmt clippy

COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

# Build compilation cache
RUN cargo build --tests
RUN cargo build --release


FROM build_cache AS builder

WORKDIR /app
COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

# As we have cargo fmt now, we can use it
RUN cargo fmt --all -- --check --color=always || (echo "Please use cargo fmt to format your code"; exit 1)
RUN cargo clippy -- -D clippy::all || (echo "Please fix clippy warnings"; exit 1)
RUN cargo rustc -- -D warnings || (echo "Warnings are not allowed"; exit 1)
RUN cargo build --tests || (echo "Please fix the failing tests"; exit 1)
RUN cargo build --release