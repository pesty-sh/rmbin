# Testing

Automated tests run in CI on temporary files. The manual list below is for real machines. It has **not** been run yet. Do not mark a box until that machine has been used, and record the OS build, filesystem, `rmbin` version, and pass or fail underneath.

Use made-up test data only, for example `EXAMPLE_TOKEN_123`. Do not start with a real token.

## What CI runs

| Target | Runner | Filesystem the tests see |
|---|---|---|
| `x86_64-apple-darwin` | `macos-15-intel` | APFS |
| `aarch64-apple-darwin` | `macos-15` | APFS |
| `x86_64-unknown-linux-musl` | `ubuntu-24.04` | ext4 |
| `aarch64-unknown-linux-musl` | `ubuntu-24.04-arm` | ext4 |
| `armv7-unknown-linux-musleabihf` | `ubuntu-24.04` via `qemu-arm` | ext4 host, armhf binary |
| `i686-unknown-linux-musl` | `ubuntu-24.04` via `qemu-i386` | ext4 host, i386 binary |
| `x86_64-pc-windows-msvc` | `windows-2025` | NTFS |

There is no macOS 13 runner. macOS 13 is manual.

Each job runs `cargo fmt`, `cargo clippy -D warnings`, `cargo test --locked`, a no-network check, and a no-C check. `cargo-deny` runs on Linux.

## Manual checklist

### macOS 13, 2014 Intel Mac, APFS

- [ ] `rmbin --version` and `rmbin --help` on the Intel binary.
- [ ] Dry run on a folder of made-up files. Bytes unchanged.
- [ ] Real run. Files gone. Banner visible.
- [ ] Symlink removed, target kept.
- [ ] `/`, home, and `/System` refused.
- [ ] Read-only file kept, report says `NOT securely deleted`.
- [ ] Optional: APFS disk image, run, then search the raw image for the test token and for `lol`. The token may still be there because of copy-on-write. The report must already have said so.

### Linux ext4

- [ ] Static musl binary (`file` says static) on x86_64.
- [ ] Same checks as above on an ext4 loop image or a disposable server.
- [ ] `grep -a` the raw image for the test token after the run. Record whether it was found.

### Windows x86_64 NTFS

- [ ] `rmbin.exe --version`.
- [ ] Dry run and real run on a temp folder.
- [ ] Symlink and, if you can, a junction: the link is not followed.
- [ ] `C:\` and `C:\Windows` refused.
- [ ] Hard-link behaviour matches the banner: 0.0.1 cannot count extra names on Windows.

### Still later

FAT32 and exFAT, files over 4 GiB, a full disk, and unplugging a USB stick mid-run are manual edge cases. They are not required to open the 0.0.1 pull request. Record them here when they are tried.

## Results

No manual row has been filled in. The owner runs this list before calling 0.0.1 tested on those devices.
