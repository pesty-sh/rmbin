# rmbin — Scope (planning document)

- Revision: 6
- Date: 2026-11-09 (Europe/London, GMT)
- Status: planning only. **No code has been written, no repo created, no dependencies installed.**
- Roadmap: `ROADMAP.md` (r1) holds the function order, releases, skill pyramid, CI and test checklists.
- Legend: **[AGREED]** explicit user requirement · **[PROPOSED]** suggested design, not yet confirmed · **[OPEN]** unresolved · **[TECHNICAL LIMIT]** a constraint, not a user preference

## 1. Purpose
[AGREED] A Rust command-line tool that permanently removes project files and folders
from a Mac and (later) servers, so details, tokens and config don't linger after a
project or DigitalOcean server is retired.

[TECHNICAL LIMIT] File-level overwrite is best effort, not certified "military-grade" sanitisation.
On SSDs, APFS (copy-on-write), snapshots and backups it cannot guarantee the old
bytes are gone (NIST SP 800-88 Rev 2: overwriting gives SSDs "very little
confidentiality protection").

## 2. Platforms
- [AGREED] First: 2014 Intel Mac running macOS 13.0, target `x86_64-apple-darwin`.
- [PROPOSED] Build floor macOS 10.12 like qrbin (Rust's Tier 2 minimum for that target).
  Only macOS 13.0 has actually been confirmed. No promise for older systems until tested.
- [AGREED] Next: Linux servers (x86_64; static musl build proposed). Later: Windows.

## 3. Input
- [AGREED] A comma- or semicolon-separated list of file and folder paths.
- [PROPOSED] Plain positional paths for names that contain `,` or `;`. Exact form:
  - `rmbin --list 'a.txt,build/;secrets/.env'` (the list is split on `,` and `;`)
  - `rmbin 'odd,name.txt' 'other;file'` (positional paths are never split)
- [PROPOSED] Always quote examples in docs so the shell doesn't break them up.

## 4. Modes
### 4.1 Permanent (default)
- [AGREED] Default is overwrite then delete, not the recycle bin.
- [AGREED] No random data. The dummy data is repeated text.
  - [PROPOSED] Default pattern `lol` (chosen in chat). Option `--pattern 101`.
  - Stuart's preference is a repeated geeky string, not random bytes.
- [PROPOSED] Write exactly the file's original byte length, never more. Then `sync_all`,
  then delete.
- [AGREED] No hidden backup is kept in permanent mode.
- [PROPOSED] One overwrite pass; no claim that extra passes overcome SSD or snapshot limits.
- Note: "no random" only means no random fill. Hashes (blake3) are still used.

### 4.2 Recycle (`--recycle`)
- [AGREED] Moves items to the system bin and **never overwrites** them.
- [AGREED] Restore on macOS via Finder (the `trash` crate cannot restore on macOS).
- [PROPOSED] If the bin is unavailable or the move fails: stop and report. **Never fall
  back to permanent delete.**

### 4.3 Safety controls
- [AGREED] `--dry-run` shows exactly what would happen and changes nothing.
- [PROPOSED] One explicit confirmation listing the exact targets before anything changes.
- [OPEN] Snapshot/backup policy. Earlier advice was to stop when proper sanitization
  isn't possible. Stuart wants a practical overwrite default. Options: warn only, or require
  a one-time `--acknowledge-best-effort` when APFS/SSD/Time Machine/snapshots are found.

### 4.4 Wipe free space (`--wipe-free-space`), extra option
- [AGREED] Opt-in extra, never part of the default run. Requested by Stuart 2026-11-09.
- [PROPOSED] Fills the volume's free space with the same repeating pattern (`lol` / `--pattern`),
  never random, syncs to disk, then deletes the fill files. Catches leftovers of files deleted
  earlier by other tools.
- [PROPOSED] Can run alone (`rmbin --wipe-free-space /Volumes/X`) or after a delete run.
- [PROPOSED] Safety: keep a small reserve so the system doesn't run out of space, show the estimated
  time, allow a clean cancel that removes the fill files, and clean up leftover fill files from an
  interrupted run on the next start (seen in Example app 1). Not supported in `--recycle`.
- [TECHNICAL LIMIT] On SSD/APFS it can't reach remapped blocks or snapshots; report says so.
- Evidence: Example apps 1 and 2 (details in the private key, `private/EXAMPLE_APPS_KEY.md`).

## 5. Workflow: plan → apply → verify [PROPOSED]
1. **plan** (reads only): splits the input, resolves paths, refuses unsafe targets, lists
   files recursively and records device, inode, size and link count. Writes a plan with no
   file contents and no secret values. Plan fingerprint is blake3. It detects edits to the
   plan but **does not prove who approved it**.
2. **apply**: needs confirmation. Refuses if the plan fingerprint or any file's identity
   changed. For each file:
   - Open it relative to a held folder handle without following links (rustix).
   - Check device, inode, size and link count again. Reject it if anything differs.
   - Overwrite from offset 0 with the pattern for the original length.
   - Run `sync_all`. An optional read-back checks only the bytes the file shows.
   - [PROPOSED, r3 from source review] Truncate to 0 + `sync_all` (SD-13), one rename to a
     same-length name made from the pattern, no random (SD-02).
   - Delete it relative to its folder handle.
   - Remove folders bottom-up once empty. Only their files are overwritten, never the
     folders themselves.
3. **verify**: confirms the targets are gone and lists what may remain.

## 6. Protections [PROPOSED]
- Refuse `/`, `$HOME`, system folders and mount points. Never cross onto another disk.
  (r3: rmbin's own macOS deny list, SD-19; skip children on another `st_dev`, SD-20.)
- Symlinks: never followed. Skipped and reported.
- Files with more than one hard link, devices, sockets and FIFOs: skipped and reported.
- Checking a path's real location once (canonicalize) is not enough. All mutation goes
  through held folder handles with identity checks to reduce path-swap risks; concurrent changes
  still require explicit handling and adversarial tests.
- `walkdir` is used only for the read-only plan listing, never to delete.
- Logs (tracing) never include environment variables, file contents or secret values.

## 7. Failure handling [PROPOSED]
- Fail closed: any write, sync, read-back or delete error stops the run.
- That file is reported as "NOT securely deleted". Never report secure success on any error.
- Report partial failure honestly. A file may already be overwritten but not deleted.
  **No rollback is promised.**
- A journal records every step so the report or a resume is exact.

## 8. Limits to state in every report
The overwrite cannot reach:
- APFS/copy-on-write copies, snapshots, Time Machine and server backups
- compressed or sparse files, and earlier versions left by apps that append or swap files
- SSD wear levelling and the drive controller moving data around
- git history, caches, editor history, logs, swap and CI artefacts

Assurance levels: A logical delete · B best-effort overwrite · C crypto-erase · D provider-managed.
[PROPOSED] No whole-disk wipes; this scope targets files and folders.

## 9. Rust libraries [PROPOSED]
| Library | Use |
|---|---|
| std (`fs::File`, `Write`, `Seek`) | overwrite to original length, `sync_all` |
| rustix | openat/statat/unlinkat on held folder handles, no-follow |
| clap | command-line flags and subcommands |
| serde + serde_json | plan and journal files |
| blake3 | plan fingerprint (not proof of approval) |
| tracing | logging with redaction |
| walkdir | read-only plan listing only |
| trash | `--recycle` (restore functions only on Linux/Windows) |
| tempfile + proptest | tests in throwaway folders |
| cargo-deb | Debian packages |

- Not needed: getrandom/rand (no random fill).
- zeroize/secrecy only if future cloud credentials are handled.
- No crate on its own solves race conditions or SSD erasure.
- No third-party secure-delete wrapper has been selected or audited. (One similarly named crate checked earlier is a garbage collector, not a secure-delete tool.)
- Pin exact versions when the project starts. Version numbers seen on 2026-11-07 are not
  recorded here as requirements.

## 10. Distribution (modelled on public pesty-sh/qrbin docs) [PROPOSED]
- Homebrew formula in `pesty-sh/homebrew-tap`, built from source with
  `cargo install --locked`. Docs say to trust just the formula, not the whole tap.
- Signed apt packages in `pesty-sh/apt` (amd64/arm64), `signed-by` keyring, plus a pin file.
- Online releases: the qrbin releases page was empty when checked on 2026-11-07. Planned:
  versioned binaries, SHA256 checksums, signatures, and the same version number across
  Cargo, the tap and apt.
- Installing, upgrading and uninstalling never delete user data. No destructive
  install/remove scripts.
- No self-update, no telemetry, offline by default.

## 11. Servers and cloud (future) [PROPOSED]
- Server-side runs use the same plan/apply/verify, with explicit approval for each host.
- Cloud features need separate opt-in networking, exact resource IDs and their own
  destructive confirmation. **No blanket account wipe.**
- Facts from DigitalOcean's docs:
  - Destroying a Droplet does not automatically delete its backups; retention depends on backup policy.
  - Snapshots and volumes stay unless you select them.
- Credentials: revoke tokens and keys at the service that issued them, as a separate step.
  Deleting files revokes nothing.
- DNS: public DNS history, certificate logs and WHOIS records can't be erased. Keep domain
  ownership and remove records that point at released IPs.

## 12. Open questions
1. Snapshot/backup acknowledgement policy (§4.3).
2. Exact list syntax: `--list` flag or a single positional argument (§3).
3. Is the default pattern `lol` final, and should `--pattern` accept any text?
4. Which Linux distros/versions to support first (qrbin's apt covers Ubuntu 18.04–24.04 and Debian 10–13).
5. Alignment with the qrbin Cursor agent. It was not found in the cloud agent list and
   was not read. Only qrbin's public READMEs were read.

## 13. Sources (checked 2026-11-07)
- https://nvlpubs.nist.gov/nistpubs/specialpublications/nist.sp.800-88r2.pdf
- https://doc.rust-lang.org/rustc/platform-support.html
- https://doc.rust-lang.org/std/fs/struct.File.html
- https://docs.rs/rustix · https://docs.rs/clap · https://docs.rs/serde_json · https://docs.rs/blake3
- https://docs.rs/tracing · https://docs.rs/walkdir · https://docs.rs/trash · https://docs.rs/tempfile
- https://docs.rs/proptest · https://docs.rs/cargo-deb
- https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap · https://docs.brew.sh/Installation
- https://wiki.debian.org/DebianRepository/UseThirdParty
- https://docs.digitalocean.com/products/droplets/how-to/destroy/
- https://github.com/pesty-sh/qrbin · https://github.com/pesty-sh/apt · https://github.com/pesty-sh/homebrew-tap

## 14. Roadmap, releases, tutorial and pure Rust (r5)
Full detail: `ROADMAP.md`.
- [AGREED] Functions are built in difficulty order, low-hanging fruit first (ROADMAP §2, F01–F36).
- [AGREED] **alpha 0.0.1**: a binary compiling on all systems (macOS x86_64 + aarch64, Linux x86_64 +
  aarch64 static musl, Windows x86_64) with the easiest functions (F01–F14: parse, plan, dry-run, `lol`
  overwrite, sync, truncate, delete, fail closed, honest report). Test volumes only.
- [AGREED] **beta 0.1.1**: more functions (F15–F25 and F28 on macOS/Linux: read-back, `--recycle`, rename,
  link/hard-link skip, one filesystem, plan/apply/verify, journal, snapshot warnings, held directory handles).
- [AGREED] Every function is built and tested in real situations: CI on real APFS/ext4/NTFS runners plus a
  manual real-device checklist (Stuart's macOS 13 Intel Mac, ext4 server, NTFS, FAT32/exFAT USB) (ROADMAP §5).
- [AGREED] The code is a beginner Rust tutorial: numbered sections, one concept per module, heavy
  explanatory comments, and a skill-pyramid dependency tree (ROADMAP §3–4).
- [AGREED] Pure Rust: no third-party C/C++ code, no build scripts compiling C, no C-wrapping `-sys` crates.
  [TECHNICAL LIMIT] OS calls still go through libSystem (macOS) / kernel32 (Windows) via std itself, and musl
  builds statically link Rust's bundled musl. Flags found: rustix uses the `libc` crate on macOS and has no
  `fs` on Windows; `trash` uses Objective-C/COM bindings; blake3 needs `features = ["pure"]` (ROADMAP §6).
- [AGREED] Global public release: GitHub Releases with checksums and signatures, Homebrew tap, apt repo,
  crates.io. [OPEN] licence choice, signing method, Apple/Windows code signing. Reviewed sources were read
  for ideas only; no code copied (licence details in the private key).
- [OPEN] Server cleanup placement: Stuart's message was cut off ("server clean up is ."). Default: after beta.

## 15. Competitor comparison docs policy [AGREED 2026-11-09, revised r6]
Public docs may say what other tools don't do and what rmbin does, without naming anyone:
- [AGREED] **Anonymised references only.** Public material never names a reviewed tool. They are "Example app 1"
  to "Example app 6", one number per reviewed source. No repo URLs, commit hashes, file paths, line numbers,
  distinctive file/function names, licences, languages or any detail that would identify an app.
- [AGREED] The identity key (Example app N → tool, source, revision, licence, evidence) lives only in
  `private/EXAMPLE_APPS_KEY.md`. `private/` and `research/` are git-ignored and never published.
- Describe behaviour generically, not intent. Never use "backdoor", "lies", "fake", "insecure", "scam" or similar.
- Prefer a behaviour table framed as "what rmbin does" (forces data to disk before delete, keeps the file if
  the overwrite fails, single honest pass, warns on APFS snapshots).
- Date claims "as reviewed in Nov 2026"; note the apps may have changed; no superiority claims beyond tested
  behaviour; state rmbin's own limits alongside; offer a correction/contact route; trademarks belong to their owners.
- [AGREED] No legal-review step (removed r6 at Stuart's request). Draft: `docs/COMPARISON.md`.

## Appendix A: What the source review found, and rmbin's decisions [PROPOSED]
Six existing tools' public code was read on 2026-11-09 (Example apps 1–6). Names, sources, revisions,
licences and file:line evidence are in the private key `private/EXAMPLE_APPS_KEY.md` and the private research
folder (`research/`, git-ignored). No code is copied; sources were read for ideas only.

| ID | Technique | Seen in (anonymised) | rmbin decision |
|---|---|---|---|
| SD-01 | One in-place overwrite, exact length, sync | Apps 1, 5 (pattern indexed by file position); App 4 writes past the end (contradicts) | **Adopt**: `lol`/`--pattern` cycle indexed by offset, never random, one pass |
| SD-23 | Full device flush on macOS | Rust std (`sync_all` = F_FULLFSYNC on Apple); App 1 uses a regular flush only | **Adopt** via `File::sync_all`; an error is fatal |
| SD-13 | Truncate after overwrite | Apps 1, 2 | **Adopt**, only after the sync |
| SD-02 | Rename before unlink | App 1 once; App 2 several times; App 6 keeps the original name (contradicts) | **Adopt**: one rename to a same-length name made from the pattern |
| SD-07 | No-follow, hard links, link-like entries | Apps 1, 2 | **Adopt**, stricter: hard-linked files are skipped entirely |
| SD-10 | Failure policy | App 1 removes the file even when the overwrite fails | **Adopt rmbin's stricter rule**: keep the file, report NOT securely deleted |
| SD-19 / SD-20 | Protected paths; stay on one filesystem | App 1 | **Adopt** (own list) |
| SD-06 | Compressed, sparse or encrypted files | App 2 refuses them; App 1 tracks disk blocks on Windows | **Adopt**: detect, skip, report (macOS UF_COMPRESSED, sparse check) |
| SD-09 | Snapshot/restore-point detection | App 2 warns about Windows restore points; none for APFS/Time Machine | **Adopt** report-only (`tmutil listlocalsnapshots`) |
| SD-11 | SSD/TRIM | App 3 skips the overwrite on SSD; App 1 sends TRIM on Linux; App 6 treats every drive as SSD (contradicts) | **Adopt** read-only diagnostics; still overwrite but label level B |
| SD-03 | Windows hidden file streams | Apps 2, 5 | Windows phase; macOS xattr/resource-fork reporting proposed |
| SD-04 / SD-12 | Tiny files kept in the file table; Windows disk-block chase | Apps 1, 2 | Windows lab only |
| SD-05 | Unused tail of the last disk block | App 2 | **[OPEN]** optional `--tail`; default stays exact length |
| SD-08 / SD-21 | Free-space wipe; orphan cleanup | Apps 1, 2; App 6's is a placeholder (contradicts) | Adopt as opt-in `--wipe-free-space` (§4.4) |
| SD-14 / SD-15 | Timestamp reset; locked files | Apps 1, 2 | Optional fixed-epoch timestamps; locked files are reported, never force-closed |
| SD-16 / SD-17 / SD-18 | Decoy overwrite; encrypt-then-overwrite; multi-pass "standards" | Apps 2, 4, 6 | **Reject** |
| SD-22 | Linux swap wipe | App 1 | Out of scope |

What none of the six does: APFS snapshot handling, F_FULLFSYNC, macOS forks/xattrs, full read-back. These are
rmbin proposals only. The §1/§8 limits still stand: none of the reviewed code reaches SSD remapping,
copy-on-write copies or backups.

## Change log
- r1 2026-11-07: first consolidated scope from the chat (modes, no-random pattern, protections,
  libraries, distribution, cloud boundaries).
- r2 2026-11-08: added Appendix A, candidate GitHub source pointers (metadata only, not verified).
- r3 2026-11-09: Appendix A replaced with source-verified evidence (6 public sources, pinned) and
  adoption decisions; §5/§6 gained proposed truncate-after, single pattern rename, deny list, one-filesystem rule.
- r4 2026-11-09: added §4.4 opt-in `--wipe-free-space` extra option (Stuart's request).
- r5 2026-11-09: added §14 roadmap/release targets/tutorial/pure-Rust requirements (see `ROADMAP.md`),
  §15 competitor comparison docs policy, server cleanup placement [OPEN] (default after beta).
- r6 2026-11-09: anonymised all reviewed tools as Example app 1–6; named evidence moved to
  `private/EXAMPLE_APPS_KEY.md` (git-ignored with `research/`); §15 policy revised (anonymised-reference rule
  [AGREED], legal-review step removed); §4.4, §9, §14 and Appendix A scrubbed.
