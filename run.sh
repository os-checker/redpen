#!/usr/bin/bash

set -eoux pipefail

# Run tests.
cargo test

# Run examples.
./examples/vec-push/run.sh
cargo run -- examples/vec-push/vec-push.rs --crate-type=lib 2>examples/vec-push/vec-push.txt
cargo run -- examples/check-panic/detected.rs --crate-type=lib 2>examples/check-panic/detected.txt

# Run cargo-redpen.
cargo install --path . --locked --debug
cd tests/vec-push
cargo redpen
