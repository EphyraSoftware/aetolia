#!/usr/bin/env bash

set -euo pipefail

cargo clean
cargo build

export RUSTFLAGS="-Cinstrument-coverage"
export LLVM_PROFILE_FILE="aetolia-%p-%m.profraw"

find --print0 . -iname "*.profraw" | xargs rm
cargo test
grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/coverage/
