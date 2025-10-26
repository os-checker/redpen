# Run

```bash
# Install redpen and cargo-redpen.
cd redpen
cargo install --path . --locked

# Run checker.
cd tests/vec-push
cargo redpen

# For your project that doesn't use the same toolchain:
cargo +nightly-2025-10-23 redpen
```

```rust
error: A possible panic spot is found.
 --> src/main.rs:3:5
  |
1 | fn main() {
  | --------- For this function.
2 |     let mut v = vec![0];
3 |     v.push(1);
  |     ^^^^^^^^^ This may panic!
```
