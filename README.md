# rmbin

rmbin removes files and folders you choose. Before deleting each file, it overwrites the file's contents once with a repeating text pattern (`lol` by default), forces that write to disk, and only then deletes the file. If any step fails, the file is kept and reported, never silently deleted. It shows exactly what it will do before it does anything, and reports what happened afterwards.

rmbin never connects to the internet. It has no updates, telemetry or tracking. The source, dependencies and release checksums are all public so you can check this yourself.

On SSDs, APFS, snapshots and backups, overwriting can't guarantee that old data is gone. rmbin tells you when that applies.

## Disclaimer / No warranty

rmbin permanently destroys data by design, and deleted files cannot be recovered. It is provided "AS IS", without warranty of any kind. To the maximum extent permitted by law, the authors and contributors accept no liability for any loss or damage, including lost or damaged data, files, software, hardware or storage devices, arising from use or misuse. Users are solely responsible for choosing targets, keeping backups, and testing with `--dry-run` first. Use at your own risk.

The MIT and Apache-2.0 licence texts already say there is no warranty. [DISCLAIMER.md](DISCLAIMER.md) is the same limit, written for this program. The authors and contributors take no responsibility for damaged files or hardware.

## Principles

1. **No network.** The program never opens a socket. There is no update check, telemetry, crash report, analytics, or phone-home. CI fails if a networking crate appears in the tree, or if the source names the standard networking API.
2. **Offline.** A built `rmbin` does not download anything. Builds use the committed `Cargo.lock` and `cargo --locked`. [docs/OFFLINE.md](docs/OFFLINE.md) explains `cargo vendor` for a machine with no network at all.
3. **Transparent.** The limits banner is printed on every run. The plan is shown before any change. The report says what happened after, including `NOT securely deleted` when a step fails. Dependencies are the list in `Cargo.lock`. Release files get SHA-256 checksums. There is no hidden mode.
4. **Honest about the disk.** One pass, the original length, a real `sync_all` (on macOS that asks for a full drive flush), then delete. If the overwrite fails, the file stays. Symlinks are not followed. The banner states the SSD, APFS, snapshot, and backup limits in the same run.

This is **0.0.1 alpha**. Use it on throwaway test files, not on the only copy of something you need. There is no warranty.

## What 0.0.1 does

For each file it will remove:

1. Open it in place, without truncating it on open, and without following a final symlink.
2. Write a repeating text pattern for exactly the original number of bytes. Byte `i` is `pattern[i % length]`. The default pattern is `lol`. `--pattern 101` uses that text instead. Random bytes are never used.
3. `sync_all`, truncate to zero, `sync_all` again.
4. Unlink the name. On macOS and Linux the parent folder is synced after that.

Folders are removed afterwards, from the deepest one upward. Only the files inside are overwritten.

A symlink is removed as a link. The thing it points at is left alone. If a folder in the middle of the path you typed is itself a symlink, rmbin refuses that path. On macOS, `/tmp` and `/var` are symlinks; pass the path that does not go through them (often under `/private`).

`--dry-run` prints the plan and changes nothing. Without `--yes`, rmbin asks you to type `yes` once for the whole run.

## What 0.0.1 does not do

These are not implemented. Passing them as flags is an error. rmbin will not pretend they ran.

- `--recycle` (move to the system bin, never overwrite)
- `--verify` (read the bytes back)
- `--wipe-free-space`
- a JSON plan, a journal, or rename-before-delete
- server or cloud cleanup

Also not in this version: Windows hard-link counts, and skipping another volume on Windows. The banner says so on Windows. On macOS and Linux, a file with more than one hard link is skipped, and a directory on a different device is not entered.

## Commands

```text
rmbin [--dry-run] [--yes] [--pattern TEXT] [--list PATHS] [PATH]...
```

| Command | Effect |
|---|---|
| `rmbin --help` | Show help. Exit 0. |
| `rmbin --version` | Print `rmbin 0.0.1`. Exit 0. |
| `--list 'a.txt,build/;c.txt'` | Split on commas and semicolons. |
| `rmbin 'odd,name.txt'` | One path. The comma is part of the name. |
| `--pattern 101` | Repeating text. Default is `lol`. |
| `--dry-run` | Print the banner and the plan. Change nothing. |
| `--yes` | Do not ask for `yes`. For scripts. |

Quote paths so the shell does not split them.

Exit codes:

| Code | Meaning |
|---|---|
| 0 | Every named path was securely deleted, or a dry run had nothing to refuse. |
| 1 | A path was refused, missing, or **NOT securely deleted**. |
| 2 | The confirmation was not `yes`, or the arguments were not understood. |

## Examples

Dry run:

```bash
rmbin --dry-run --list 'notes.txt,build/'
```

A name that contains a comma:

```bash
rmbin --yes 'odd,name.txt'
```

Your own pattern, after reading the plan and typing `yes`:

```bash
rmbin --pattern 101 secret.txt
```

## Protected paths

rmbin refuses filesystem roots, your home directory itself (a file inside it is allowed), mount points you name, and system directories such as `/etc`, `/usr`, `/System/Library`, and `C:\Windows`. The exact list is the tables in [`src/protect.rs`](src/protect.rs). A refusal changes nothing and exits 1.

## Build

Requirements: Rust 1.85 (see [`rust-toolchain.toml`](rust-toolchain.toml)). The only runtime crate is `clap`, pinned in [`Cargo.toml`](Cargo.toml) and [`Cargo.lock`](Cargo.lock).

```bash
cargo build --release --locked
cargo test --locked
```

`make` builds the release binary and leaves it at `target/release/rmbin`. It does not copy it onto `PATH`. The program does not install or uninstall itself, and install docs do not delete your files.

Targets the release workflow builds. One macOS build is not made per macOS version. The Intel binary is stamped 10.12 and the Apple silicon binary is stamped 11.0. `lipo` joins those two into one universal binary. There is no `.app`.

| Archive | Minimum system |
|---|---|
| `x86_64-apple-darwin` | macOS 10.12 on Intel. The owner's first machine is macOS 13. |
| `aarch64-apple-darwin` | macOS 11.0 on Apple silicon |
| `rmbin-universal-apple-darwin.tar.gz` | The two slices above, in one file |
| `x86_64-unknown-linux-musl` | Static. `.deb` arch `amd64`. Ubuntu 14.04 / Debian 8 or newer |
| `aarch64-unknown-linux-musl` | Static. `.deb` arch `arm64`. Same floor |
| `armv7-unknown-linux-musleabihf` | Static. `.deb` arch `armhf`. Same floor |
| `i686-unknown-linux-musl` | Static. `.deb` arch `i386`. Same floor |
| `x86_64-pc-windows-msvc` | Windows 10 or Windows Server 2016, 64-bit |

CI runs on current GitHub images, not on Ubuntu 14.04 or macOS 10.12. A static binary does not need those systems' C libraries. That is not the same as having booted them. See [docs/TESTING.md](docs/TESTING.md) and [docs/RELEASING.md](docs/RELEASING.md).

## Checksums and releases

The tag workflow [`.github/workflows/release.yml`](.github/workflows/release.yml) runs when a `v*` tag is pushed. It uploads one archive per target and a `SHA256SUMS` file. Nothing is signed yet. Signing is still an open choice.

```bash
sha256sum -c SHA256SUMS
```

On Windows, `Get-FileHash file -Algorithm SHA256` and compare the digest to `SHA256SUMS`.

Publishing steps for the owner are in [docs/PUBLISHING.md](docs/PUBLISHING.md). This tree does not publish a tag, a GitHub Release, Homebrew, apt, or crates.io by itself.

## Learn the code

[`docs/TUTORIAL.md`](docs/TUTORIAL.md) is the reading order. [`docs/SKILL_PYRAMID.md`](docs/SKILL_PYRAMID.md) is the skill tree. The source files are numbered in their headers, one idea each.

[`docs/COMPARISON.md`](docs/COMPARISON.md) is a draft. It does not name other programs. It is not a tested claim yet. Do not treat it as part of the 0.0.1 release notes.

## Privacy and security

[PRIVACY.md](PRIVACY.md) states what the program does not collect. [SECURITY.md](SECURITY.md) states the limits and how to report a problem.

## Licence

Provisionally **MIT OR Apache-2.0**, which is the usual dual licence for Rust crates. The owner has not confirmed this choice. See [LICENSE](LICENSE), [LICENSE-MIT](LICENSE-MIT), and [LICENSE-APACHE](LICENSE-APACHE).
