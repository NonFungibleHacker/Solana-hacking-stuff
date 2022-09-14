#!/bin/bash
cargo build-bpf --manifest-path=./Cargo.toml --bpf-out-dir=./target/so
solana program deploy ./target/so/level1.so
RUST_BACKTRACE=1 cargo run --manifest-path=./native/Cargo.toml --target-dir=./target/
#RUST_BACKTRACE=1 cargo run --bin native
