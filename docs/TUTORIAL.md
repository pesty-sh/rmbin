# Tutorial

`rmbin` is written so a new Rust programmer can read it from top to bottom. Each file starts with a header: what you will learn, what it builds on, and which function numbers from the roadmap it covers.

Rust module names cannot start with a digit, so the numbers live in the headers and in this list.

## Reading order

| Order | File | Idea | Roadmap |
|---|---|---|---|
| 00 | [`src/main.rs`](../src/main.rs) | `fn main` and an exit code | F01 |
| 01 | [`src/error.rs`](../src/error.rs) | `enum`, `Result`, `?` | F04 |
| 02 | [`src/cli.rs`](../src/cli.rs) | a struct and clap's derive macro | F01 |
| 03 | [`src/input.rs`](../src/input.rs) | `&str`, `split`, `Vec` | F02 |
| 04 | [`src/pattern.rs`](../src/pattern.rs) | bytes and `%` | F03 |
| 05 | [`src/classify.rs`](../src/classify.rs) | `symlink_metadata`, `match` | F05 |
| 06 | [`src/protect.rs`](../src/protect.rs) | path parts and the deny list | F07 |
| 07 | [`src/plan.rs`](../src/plan.rs) | walk a tree, bottom up | F08, F09 |
| 08 | [`src/confirm.rs`](../src/confirm.rs) | one line from stdin | F10 |
| 09 | [`src/overwrite.rs`](../src/overwrite.rs) | `OpenOptions`, `Write`, `sync_all` | F11, F12 |
| 10 | [`src/remove.rs`](../src/remove.rs) | delete names after the overwrite | F13 |
| 11 | [`src/report.rs`](../src/report.rs) | the banner and `NOT securely deleted` | F06, F14 |
| 12 | [`src/identity.rs`](../src/identity.rs) | the id card, read only | F19, F28 |
| 13 | [`src/readback.rs`](../src/readback.rs) | the later read-back, as a sentence | F15 |
| 14 | [`src/rename_name.rs`](../src/rename_name.rs) | the later file name, not created | F18 |
| 15 | [`src/sparse.rs`](../src/sparse.rs) | sparse or compressed, read only | F24 |
| 16 | [`src/snapshot.rs`](../src/snapshot.rs) | filesystem and snapshot warning | F25 |
| 17 | [`src/disk.rs`](../src/disk.rs) | the kernel rotational flag | F30 |
| 18 | [`src/remain.rs`](../src/remain.rs) | what may still hold old bytes | F23 |
| 19 | [`src/redact.rs`](../src/redact.rs) | a report line with no file bytes | F17 |
| 20 | [`src/core.rs`](../src/core.rs) | the dry-run, step by step | the plan |
| — | [`src/run.rs`](../src/run.rs) | call Section 20 | — |
| — | [`src/lib.rs`](../src/lib.rs) | the module map | — |

Sections 09 and 10 are the overwrite and the delete. **Section 20 does not call them.** Recycle, a JSON plan, a journal, and free-space fill are later. The roadmap snapshot is [`planning/ROADMAP.md`](planning/ROADMAP.md). [`SKILL_PYRAMID.md`](SKILL_PYRAMID.md) is the same idea drawn as a tree.

Rules in the code:

- No `unsafe`.
- No `unwrap` in the program itself. Tests may use `expect`.
- One concept per file. Comments say why.

## Try it yourself

Use a made-up token, never a real one. Work in a new folder.

1. **Pattern.** In `pattern.rs`, the tests check that byte 3 of `lol` is `l`. Add a test for `--pattern ab` at offset 5. You want `b`, because `5 % 2 == 1`.
2. **List splitting.** Run `cargo test list_is_split`. Then try `rmbin --dry-run --list 'a.txt,b.txt'` on two real files.
3. **Dry run.** Put `EXAMPLE_TOKEN_123` in `notes.txt`. Run `rmbin --dry-run notes.txt`. The file must still contain the token. The banner must mention SSDs and APFS.
4. **No delete.** Run `rmbin notes.txt`. The plan is printed. The file is still there. Exit code 2. `--yes` does not change that.
5. **Symlink.** `ln -s notes.txt link`. `rmbin --dry-run link` says it would delete the link only. `notes.txt` and `link` both stay.
6. **Read-only.** `chmod a-w notes.txt`, then `rmbin --dry-run notes.txt`. The bytes are still there, because this version does not open the file for writing. `chmod u+w notes.txt` when you are done.

`cargo test` runs these ideas on temporary folders so you do not have to risk a real file.
