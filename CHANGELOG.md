# Changelog

## 0.0.1 — alpha

Not released. The owner publishes the tag `v0.0.1` after review. See [docs/PUBLISHING.md](docs/PUBLISHING.md).

- One command: overwrite with a repeating text pattern (`lol`, or `--pattern`), `sync_all`, truncate, `sync_all`, then delete.
- `--list` splits on commas and semicolons. Positional paths do not.
- `--dry-run` changes nothing. One `yes` confirmation, skipped with `--yes`.
- Symlinks are not followed. Protected paths are refused. The run fails closed.
- The limits banner is printed on every run.
- No network access, no telemetry, and no extra crates beyond `clap`.
- Not in this version: `--recycle`, `--verify`, `--wipe-free-space`, JSON plans, rename-before-delete, and server cleanup.

The licence is provisionally MIT OR Apache-2.0 until the owner confirms it.
