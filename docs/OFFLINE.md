# Building without a network

The `rmbin` binary does not use the network. `cargo build` does, the first time, to download the crates named in `Cargo.lock`. After that, `--offline` works.

## Normal build

```bash
cargo build --release --locked
cargo test --locked
```

`--locked` refuses to change `Cargo.lock`. CI uses it on every job.

## No network at all

On a machine that already has the crates, vendor them once while you still have a network:

```bash
cargo vendor vendor
```

`vendor/` is gitignored. It is a copy of the crates, not a second dependency list. Then create `.cargo/config.toml` (also not committed, so normal CI can still reach crates.io):

```toml
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
```

Then:

```bash
cargo build --release --locked --offline
```

Delete `.cargo/config.toml` when you want the normal crates.io setup again. Do not commit a config that forces the vendor directory, or a checkout without `vendor/` will not build.

## What is pinned

- Rust 1.85.0 in `rust-toolchain.toml` and `rust-version` in `Cargo.toml`.
- Exact `clap` 4.5.53. The `wrap_help` feature is off so the tree does not pull `terminal_size` or `rustix`.
- `Cargo.lock` is committed.

Release builds use `lto`, one codegen unit, and `panic = abort`. That makes builds more repeatable. It does not promise bit-for-bit identical binaries across machines. The release notes should record `rustc --version`.
