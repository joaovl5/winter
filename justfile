#!/usr/bin/env just --justfile
set positional-arguments

@_:
  just --list

RUST_LOG := "trace"

run *args:
  RUST_LOG={{RUST_LOG}} cargo run --bin winter-cli -- "$@"




check: check-cargo check-clippy check-fmt

check-cargo:
  RUSTFLAGS="-Dwarnings" cargo check

check-clippy:
  cargo clippy -- \
    -W clippy::pedantic \
    -W clippy::correctness \
    -W clippy::suspicious \
    -W clippy::cargo

check-fmt:
  cargo fmt --check

fix: fix-cargo fix-clippy fix-fmt

fix-cargo:
  cargo fix --allow-dirty

fix-clippy:
  cargo clippy --fix --allow-dirty -- \
    -W clippy::pedantic \
    -W clippy::correctness \
    -W clippy::suspicious \
    -W clippy::cargo

fix-fmt:
  cargo fmt

test:
  cargo nextest run
