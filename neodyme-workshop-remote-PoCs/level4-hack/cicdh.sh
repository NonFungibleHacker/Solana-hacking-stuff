#!/bin/bash

# This script is for quick building & deploying of the program.
# It also serves as a reference for the commands used for building & deploying Solana programs.
# Run this bad boy with "bash cicd.sh" or "./cicd.sh"

cargo build-bpf --manifest-path=./Cargo.toml --bpf-out-dir=./target/so
solana program deploy ./target/so/level4.so
solana program deploy ./target/so/myspl.so
RUST_BACKTRACE=1 cargo run --bin tok
#RUST_BACKTRACE=1 cargo run --bin poc
#IF error, replace the program ID in the poc.rs with the correct
#solana program show --programs
