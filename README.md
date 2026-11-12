# rmbin

rmbin is a careful plan for `rm` on Linux. You name files and folders. It prints what a later version would do: overwrite each file once with a repeating text pattern (`lol` by default), read those bytes back, rename the file, and only then delete it. **This version does not do any of that.** It does not overwrite, rename, move, or delete. `--yes` is ignored. The bin and free-space fill are later, because they change the disk.

rmbin never connects to the internet. It has no updates, telemetry or tracking. The source, dependencies and release checksums are all public so you can check this yourself.

On SSDs, APFS, snapshots and backups, overwriting can't guarantee that old data is gone. rmbin tells you when that applies.

## Disclaimer / No warranty

rmbin permanently destroys data by design, and deleted files cannot be recovered. It is provided "AS IS", without warranty of any kind. To the maximum extent permitted by law, the authors and contributors accept no liability for any loss or damage, including lost or damaged data, files, software, hardware or storage devices, arising from use or misuse. Users are solely responsible for choosing targets, keeping backups, and testing with `--dry-run` first. Use at your own risk.

The MIT and Apache-2.0 licence texts already say there is no warranty. [DISCLAIMER.md](DISCLAIMER.md) is the same limit, written for this program. The authors and contributors take no responsibility for damaged files or hardware.

## Principles

1. **No network.** The program never opens a socket. There is no update check, telemetry, crash report, analytics, or phone-home. CI fails if a networking crate appears in the tree, or if the source names the standard networking API.
2. **Offline.** A built `rmbin` does not download anything. Builds use the committed `Cargo.lock` and `cargo --locked`. [docs/OFFLINE.md](docs/OFFLINE.md) explains `cargo vendor` for a machine with no network at all.
3. **Transparent.** The limits banner is printed on every run. The plan is shown, and this version stops there. A refused path is `NOT securely deleted`. Dependencies are the list in `Cargo.lock`. Release files get SHA-256 checksums. There is no hidden mode.
4. **Honest about the disk.** A later version would write one pass, the original length, then a real `sync_all` (on macOS that asks for a full drive flush), then delete. This version does not write. If that later overwrite fails, the file is meant to stay. Symlinks are not followed. The banner states the SSD, APFS, snapshot, and backup limits in the same run.

This is **0.0.10**, and it is dry-run only. There is no warranty. Do not expect a file to disappear.

Put the files you want to plan in the `sandbox` folder next to `Cargo.toml`. `rmbin` refuses every other path. It also refuses to run as root, so it cannot be used to wipe this drive. The `sandbox` folder itself is not a target.

## What 0.0.9 does

It prints a plan. The plan is what a later version would do, in this order:

1. Refuse protected paths, and refuse to walk through a symlink in the middle of a path.
2. List the files. A symlink is named, not followed. A file with more than one hard link is skipped. A directory on another device is not entered.
3. Skip a sparse or compressed file, and say so.
4. Say which filesystem the path is on, and warn that snapshots and backups are not deleted.
5. On Linux, print the kernel's rotational flag when `/sys` has one. That flag can be wrong for a virtual disk.
6. For each ordinary file, show the id card (length, links, device, inode) and the name a later rename would use. The name is the pattern repeated to the same length. It is not created.
7. Say that a later version would read the visible bytes back, and that this version does not read file contents.

`--dry-run` is the same plan, and it exits 0 when nothing would be skipped. Without `--dry-run` the plan is the same and the exit code is 2, so a script cannot treat the run as a successful delete.

## What this version does not do

These are the dangerous points. They are left for a later version. The command does not call them.

- overwrite, truncate, rename, or delete
- `--recycle` (move to the system bin)
- `--verify` as a performed read-back (the plan only describes it)
- `--wipe-free-space`
- a JSON plan file, a journal, or a log file
- server or cloud cleanup

Passing `--recycle`, `--verify`, or `--wipe-free-space` is an error. Nothing is changed. `--yes` is ignored.

On macOS, `/tmp` and `/var` are symlinks. A path through them is refused. Pass the path that does not go through them (often under `/private`).

Windows still cannot count extra hard-link names, and it cannot yet tell that a folder is on another volume. The banner says so. This version does not write on Windows either.

## Commands

```text
rmbin [--dry-run] [--yes] [--pattern TEXT] [--list PATHS] [PATH]...
```

| Command | Effect |
|---|---|
| `rmbin --help` | Show help. Exit 0. |
| `rmbin --version` | Print `rmbin 0.0.10`. Exit 0. |
| `--list 'a.txt,build/;c.txt'` | Split on commas and semicolons. |
| `rmbin 'odd,name.txt'` | One path. The comma is part of the name. |
| `--pattern 101` | Repeating text. Default is `lol`. |
| `--dry-run` | Print the banner and the plan. Exit 0 when nothing would be skipped. |
| `--yes` | Ignored. This version cannot delete. Exit 2. |

Quote paths so the shell does not split them.

Exit codes:

| Code | Meaning |
|---|---|
| 0 | A dry run had nothing to refuse and nothing a later version would skip. |
| 1 | A path was refused or missing, or a dry run found a path a later version would skip. |
| 2 | You asked to delete (no `--dry-run`), or the arguments were not understood. Nothing was deleted. |

## Examples

Dry run:

```bash
rmbin --dry-run --list 'notes.txt,build/'
```

A name that contains a comma:

```bash
rmbin --yes 'odd,name.txt'
```

Your own pattern, still only a plan:

```bash
rmbin --dry-run --pattern 101 secret.txt
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
| `x86_64-apple-darwin` | macOS 10.12 on Intel. The owner's first machine is macOS 13. Built by the optional Intel workflow, not by pull-request CI. |
| `aarch64-apple-darwin` | macOS 11.0 on Apple silicon. Built by the optional Apple silicon workflow. |
| `rmbin-universal-apple-darwin.tar.gz` | Optional third step: `lipo` after both Mac tarballs are on the release. Not an `.app`. |
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

[`docs/COMPARISON.md`](docs/COMPARISON.md) is a draft. It does not name other programs. It describes later behaviour. This version only prints a plan.

## Privacy and security

[PRIVACY.md](PRIVACY.md) states what the program does not collect. [SECURITY.md](SECURITY.md) states the limits and how to report a problem.

## Licence

Provisionally **MIT OR Apache-2.0**, which is the usual dual licence for Rust crates. The owner has not confirmed this choice. See [LICENSE](LICENSE), [LICENSE-MIT](LICENSE-MIT), and [LICENSE-APACHE](LICENSE-APACHE).
