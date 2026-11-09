# rmbin release notes

Alpha. Overwrite is best effort on SSDs, APFS, snapshots, and backups. The binaries are not signed. There is no warranty. Read DISCLAIMER.md. Deletion is permanent.

## Minimum systems

These are the floors the binaries are built for. CI does not boot every old release.

| File | Runs on |
|---|---|
| `rmbin-vVERSION-x86_64-apple-darwin.tar.gz` | macOS 10.12 or newer, Intel |
| `rmbin-vVERSION-aarch64-apple-darwin.tar.gz` | macOS 11.0 or newer, Apple silicon |
| `rmbin-universal-apple-darwin.tar.gz` | Both of the above, one file. Not an `.app` |
| `rmbin-vVERSION-x86_64-unknown-linux-musl.tar.gz` and `rmbin_VERSION-1_amd64.deb` | Linux amd64. Static binary. Ubuntu 14.04 / Debian 8 or newer |
| `rmbin-vVERSION-aarch64-unknown-linux-musl.tar.gz` and `rmbin_VERSION-1_arm64.deb` | Linux arm64. Same floor |
| `rmbin-vVERSION-armv7-unknown-linux-musleabihf.tar.gz` and `rmbin_VERSION-1_armhf.deb` | Linux armhf (32-bit ARM). Same floor |
| `rmbin-vVERSION-i686-unknown-linux-musl.tar.gz` and `rmbin_VERSION-1_i386.deb` | Linux i386. Same floor |
| `rmbin-vVERSION-x86_64-pc-windows-msvc.zip` | Windows 10 or Windows Server 2016, 64-bit |

The `.deb` files are gzip-compressed so older dpkg can install them. They do not depend on glibc.

## Checksums

Download `SHA256SUMS` next to the archives.

```bash
sha256sum -c SHA256SUMS
```

On Windows, `Get-FileHash file -Algorithm SHA256` and compare the digest to the matching line.

Windows on ARM is not in this release. A 64-bit x86 Windows binary does not run on Windows ARM unless the machine emulates it.
