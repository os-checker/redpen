#!/usr/bin/bash

set -eoux pipefail

cargo run --example vec-push-mir -- examples/vec-push/vec-push.rs 2>examples/vec-push/MIR-Body.txt

cd examples/vec-push

rustc vec-push.rs -Zdump-mir=main -Zdump-mir-graphviz
rm vec-push

# Need graphviz installed.
# dot -Tsvg mir_dump/vec_push.main.-------.renumber.0.dot -o main.svg
