# Contributing

rmbin permanently deletes data. Read [DISCLAIMER.md](DISCLAIMER.md) before you run it, including in tests. Use temporary directories and made-up tokens. Do not point a development build at a real home directory or a system folder.

## Build and test

Rust 1.85. See [rust-toolchain.toml](rust-toolchain.toml).

```bash
cargo test --locked
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
./packaging/check-no-network.sh
./packaging/check-no-c.sh
```

`cargo --locked` refuses to change `Cargo.lock`. The running program has no network access. Do not add a crate that opens sockets. CI rejects those names, and it rejects C build scripts.

## What belongs in 0.0.9

A dry-run plan. Do not wire the command to overwrite, rename, recycle, fill free space, or delete. Those are a later version. Sections 09 and 10 stay in the tree so the later version has a place to start. Do not pretend a stub ran.

## Reports

Open an issue with [the template](.github/ISSUE_TEMPLATE/bug_report.md). Do not paste real secrets or file contents.

## Licence

The tree is provisionally MIT OR Apache-2.0 until the owner confirms the choice. A contribution is offered under those same terms.
