# Security

`rmbin` overwrites a file with a repeating text pattern, asks the operating system to flush that write, and then deletes the name. Assurance level B means best-effort overwrite. It is not a certified wipe.

## What a failure looks like

If opening, writing, syncing, or truncating fails, the name is kept and the report says `NOT securely deleted`. The run stops. Later paths are `not attempted`. Nothing is restored. A file that was already overwritten stays overwritten. The message says when the bytes may already have changed, and when the open failed before any write.

If the name cannot be deleted after a successful overwrite, the report says that too. The file may already be empty. It is still `NOT securely deleted`.

## Limits

Overwriting cannot reach:

- copies made by APFS or another copy-on-write filesystem
- snapshots, Time Machine, and server backups
- blocks remapped by SSD wear levelling or the drive controller
- compressed or sparse copies, or older bytes left when a program swaps a file
- git history, caches, editor history, logs, swap, and CI artefacts

NIST SP 800-88 Rev 2: overwriting gives SSDs very little confidentiality protection. The banner in every run says this.

## Links

A symlink that you name is removed. The target is not opened. A symlink in the middle of a path is refused, so the run does not walk through it. On macOS, `/tmp` and `/var` are symlinks.

On macOS and Linux, `open` of the final component uses `O_NOFOLLOW`. A swap that replaces the file with a symlink between the plan and the open should fail the open. That is not a full race-proof design. Held directory handles are later work. A determined swap can still win on 0.0.1.

Files with more than one hard link are skipped on macOS and Linux. Overwriting them would change every name. Windows 0.0.1 cannot read that count with safe standard-library calls, so a second name would be overwritten too. The Windows banner says this.

On macOS and Linux, a directory on a different device is not entered. Windows 0.0.1 does not have that check yet.

## Protected paths

The deny list is the source in `src/protect.rs`: roots, your home directory itself, system directories, mount points, and macOS volume roots. A child of your home directory is allowed. Refusals happen before any delete.

## Network

The program does not use the standard networking API. CI rejects networking crates and any source mention of that API. On Windows, the Rust standard library is linked as a whole and the import list may name the system socket library even though this program never calls it. CI still rejects socket use in our source.

Release archives are checksummed with SHA-256 and are not signed. Signature keys are an open decision. Verify `SHA256SUMS` as described in the README.

## Reporting a problem

Open an issue at <https://github.com/pesty-sh/rmbin/issues>. Do not include real secrets, real tokens, or file contents. A stand-in value such as `EXAMPLE_TOKEN_123` is enough.

There is no bug bounty and no security email alias yet. The owner can add one.
