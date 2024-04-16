#!/usr/bin/env just --justfile

export RUST_BACKTRACE := "full"
export SKIP_WASM_BUILD := "1"
export RUST_BIN_DIR := "target/x86_64-unknown-linux-gnu"
export TARGET := "x86_64-unknown-linux-gnu"
export RUSTV := "nightly-2024-03-05"
export RELEASE_NAME := "development"

default:
  @echo "Running all tasks..."
  fmt
  check
  test
  benchmarks
  clippy
  fix

fmt:
  @echo "Running cargo fmt..."
  cargo +{{RUSTV}} fmt --check --all

check:
  @echo "Running cargo check..."
  cargo +{{RUSTV}} check --workspace

test:
  @echo "Running cargo test..."
  cargo +{{RUSTV}} test --workspace

benchmarks:
  @echo "Running cargo test with benchmarks..."
  cargo +{{RUSTV}} test --workspace --features=runtime-benchmarks

clippy:
  @echo "Running cargo clippy..."
  cargo +{{RUSTV}} clippy -- -D clippy::panic \
                            -D clippy::todo \
                            -D clippy::unimplemented
fix:
  @echo "Running cargo fix..."
  cargo +{{RUSTV}} fix --workspace
  git diff --exit-code || (echo "There are local changes after running 'cargo fix --workspace' ❌" && exit 1)