# Changelog

The binary version is **0.0.9**. It only prints a plan. 0.0.2 through 0.0.9 are the safe steps below. They were not tagged separately. Deleting, renaming, the bin, and filling free space are a later version.

## 0.0.9 — what may remain, still a dry run

- The plan ends with what a later overwrite would still miss: snapshots, backups, SSD remapped blocks, and copies in git, caches, and logs.
- Report lines are built from a label and a path. There is no argument for file bytes, and no log file is written (`src/remain.rs`, `src/redact.rs`).

## 0.0.8 — disk flag, read only

- On Linux, when the mount device is under `/dev`, the plan prints the kernel `rotational` flag from `/sys`. It says a virtual disk can report the flag wrong. Nothing is written (`src/disk.rs`).

## 0.0.7 — filesystem note, read only

- The plan says which filesystem holds the path, from `/proc/mounts` on Linux. Snapshots and backups are not deleted. btrfs, zfs, and overlay get one extra sentence (`src/snapshot.rs`).

## 0.0.6 — sparse and compressed, read only

- A file whose allocated blocks are smaller than its length is marked skip in the plan. On macOS the compressed flag is the same kind of skip. The file is not opened for writing (`src/sparse.rs`).

## 0.0.5 — the later rename name, not created

- The plan shows a same-length name made from the pattern. Slashes and Windows reserved names are adjusted. The name is not created (`src/rename_name.rs`).

## 0.0.4 — read-back, described only

- The plan says a later version would compare visible bytes with the pattern and would not print the bytes. This version does not read file contents (`src/readback.rs`).

## 0.0.3 — id card, read only

- The plan prints length, link count, device, and inode. It does not print file bytes (`src/identity.rs`).

## 0.0.2 — the command stops before any write

- `rmbin` no longer overwrites or deletes. `--yes` is ignored. Without `--dry-run` the exit code is 2 so a script cannot treat the plan as a successful delete.
- `--recycle`, `--verify`, and `--wipe-free-space` are still unknown flags. They are the dangerous points and they stay later.
- Sections 09 and 10 remain in the tree for the tutorial. The command does not call them.

## 0.0.1 — alpha

Not released. The owner publishes the tag `v0.0.1` after review. See [docs/PUBLISHING.md](docs/PUBLISHING.md).

- One command: overwrite with a repeating text pattern (`lol`, or `--pattern`), `sync_all`, truncate, `sync_all`, then delete.
- `--list` splits on commas and semicolons. Positional paths do not.
- `--dry-run` changes nothing. One `yes` confirmation, skipped with `--yes`.
- Symlinks are not followed. Protected paths are refused. The run fails closed.
- The limits banner is printed on every run.
- No network access, no telemetry, and no extra crates beyond `clap`.
- Not in this version: `--recycle`, `--verify`, `--wipe-free-space`, JSON plans, rename-before-delete, and server cleanup.
- Disclaimer: deletion is permanent, the program is "AS IS", and the authors accept no liability for lost data or damaged hardware. See DISCLAIMER.md. `--help` points at it. The confirmation prompt says the deletion is permanent and that typing `yes` accepts the risk.

- Release pipeline, not yet run: a `v*` tag builds four static musl binaries and their gzip `.deb` packages (amd64, arm64, armhf, i386), macOS Intel and Apple silicon archives, a universal macOS binary made with `lipo`, a Windows x86_64 zip, and `SHA256SUMS`. See [docs/RELEASING.md](docs/RELEASING.md).

The licence is provisionally MIT OR Apache-2.0 until the owner confirms it.
