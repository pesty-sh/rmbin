# How rmbin compares (DRAFT — not for publication until the checklist below is done)

> Status: draft, Nov 2026. Follows the comparison docs policy (`ROADMAP.md` §9, `RMBIN_SCOPE.md` §15).
> rmbin itself is not built yet. Every "rmbin" entry is a *design goal* until tested, and must be re-worded
> to "tested on <OS/filesystem>, <date>" before publishing.

## About this page
- We read the public source code of six existing secure-delete tools to learn what works and what to
  avoid. They are referred to only as **Example app 1 – Example app 6**. We don't name them.
- Everything below describes **behaviour observed in their code, as reviewed in Nov 2026**. It says nothing
  about anyone's intent. These projects may have changed since then.
- rmbin contains no code from any of them; they were read for ideas only.
- All trademarks belong to their owners.
- Corrections: if you think something here is inaccurate, open an issue at <REPO_URL>/issues or email
  <CONTACT_EMAIL>. We'll check and correct it promptly. [placeholders, OPEN]

## What rmbin does
| Behaviour | rmbin (design goal → must be tested) | Seen in the example apps (as reviewed in Nov 2026) |
|---|---|---|
| Forces written data onto the drive before deleting | Uses Rust's `File::sync_all`, which on macOS asks the drive for a full flush | Example app 1 uses a regular flush, which on macOS doesn't request the full drive flush. Example app 4's flush doesn't ask the OS to write through to disk. Example apps 5 and 6 do flush to disk |
| Writes exactly the file's length | Overwrites from the start to the original length, never more | Example app 1 writes exactly the file size. Example app 4 writes whole buffers, which can make the file larger |
| If the overwrite fails | Keeps the file and reports it as "NOT securely deleted" | Example app 1 still removes the file and logs a warning |
| Number of passes | One pass of a repeating text pattern (`lol` by default), no multi-pass labels | Example app 4's multi-pass presets repeat the same buffer. Example app 6's longest preset mixes fixed and random passes. Example app 2 implements a full multi-pass table |
| Renaming before delete | One rename to a same-length name made from the pattern; the new name never contains the old one | Example app 1 renames once. Example apps 2 and 3 rename several times. Example app 6's new names begin with the original name |
| SSD handling | Still overwrites, labels the result "best effort" and explains SSD limits | Example app 3 does a plain delete when it detects an SSD. Example app 6 treats every drive as an SSD |
| APFS snapshots / Time Machine | Warns when local snapshots exist (read-only check) | Not seen in any of the six |
| Links and hard links | Never follows links; skips files with more than one hard link | Example app 1 refuses to follow links and removes hard-linked files without overwriting them. Example app 2 skips link-like entries |
| Free-space wipe | Opt-in `--wipe-free-space` (planned, later) | Example apps 1 and 2 implement it. Example app 6's version doesn't write anything yet. Example app 5 leaves part of the free space unwritten |
| Randomness | None; repeating text by design | Most of the example apps use random data in some modes |

What some example apps do that rmbin doesn't (yet): wipe Windows hidden file streams, wipe the unused tail
of a file's last disk block, track a file's disk blocks on Windows, wipe Linux swap, and send TRIM on Linux.

## rmbin's own limits (always shown with this table)
Overwriting files can't reach copy-on-write copies (APFS), snapshots, Time Machine or server backups,
SSD wear-levelling remaps, or copies in git history, caches and logs. See NIST SP 800-88 Rev 2.

## Before publishing (checklist)
- [ ] Re-check every claim against the private evidence key (not public)
- [ ] Check whether the example apps have changed since Nov 2026; drop or reword any claim that no longer holds
- [ ] Replace every rmbin "design goal" with tested results (OS, filesystem, version, date)
- [ ] Behaviour only: no wording about intent or quality, and no detail that would identify an app
- [ ] Fill in the correction contact
