# How a release is built

This page is the map of [`.github/workflows/release.yml`](../.github/workflows/release.yml). Nothing here runs until somebody pushes a tag. Do not push a tag until you want a public GitHub Release.

GitHub Actions is the robot that runs those steps on GitHub's computers. For a **public** repository, GitHub does not bill the standard runner minutes. This repository is public. A private repository has a monthly allowance and can be charged after that. A full rmbin release uses about ten jobs. Each one is a few minutes. That is normal for a public project. The minutes are still shared machines, so the workflow does not rebuild the same thing twice.

## Cut a release

On `main`, after the version in `Cargo.toml` is the version you want:

```bash
git tag v0.0.1 && git push --tags
```

`git tag v0.0.1` creates a name that points at the current commit. `git push --tags` sends **every** tag in your clone, not only the new one. If you have old local tags you do not want on GitHub, push just this tag instead:

```bash
git push origin v0.0.1
```

The workflow only starts for tags that look like `v0.0.1` (the pattern is `v*`). It checks that the tag, without the `v`, matches `version` in `Cargo.toml`. `v0.0.1` matches `0.0.1`. A mismatch fails the job before it uploads anything.

Pushing the tag creates the GitHub Release when the jobs succeed. There is no separate "publish" button in this repo.

## What happens, in order

### 1. Tag push

GitHub sees the new tag and starts the Release workflow. Pull requests do not start it.

### 2. Matrix builds

A matrix is one job description repeated with different settings. Each setting is one target:

| Rust target | Runner | Archive |
|---|---|---|
| `x86_64-apple-darwin` | optional Intel workflow, not this matrix | See section 4 |
| `aarch64-apple-darwin` | optional Apple silicon workflow, not this matrix | See section 4 |
| `x86_64-unknown-linux-musl` | Ubuntu x86_64 | `.tar.gz` plus `amd64` `.deb` |
| `aarch64-unknown-linux-musl` | Ubuntu arm64 | `.tar.gz` plus `arm64` `.deb` |
| `armv7-unknown-linux-musleabihf` | Ubuntu x86_64, cross-linked | `.tar.gz` plus `armhf` `.deb` |
| `i686-unknown-linux-musl` | Ubuntu x86_64, cross-linked | `.tar.gz` plus `i386` `.deb` |
| `x86_64-pc-windows-msvc` | Windows | `.zip` |

There is one macOS build per CPU family, not one per macOS version. The Intel binary asks for macOS 10.12. The Apple silicon binary asks for 11.0, which is the oldest system that slice can load. Windows on ARM is not built. The hosted Windows image does not reliably include the ARM64 MSVC libraries, so that target is a comment in the workflow rather than a job that would fail.

Linux binaries are static musl. They do not need the system's glibc, which is why the intended floor is Ubuntu 14.04 and Debian 8. CI itself runs on Ubuntu 24.04. armv7 and i686 are linked with `rust-lld` from the Rust toolchain (`packaging/rust-lld.sh`), not with a C cross compiler. The rmbin binary stays pure Rust. `clap` is still the only runtime crate.

### 3. `.deb` packages

On each Linux leg, after the binary exists, the job installs `cargo-deb` and runs:

```bash
cargo deb --no-build --compress-type gzip --target <triple>
```

`--no-build` packs the binary just produced. It does not compile a second time. `--compress-type gzip` puts `data.tar.gz` inside the `.deb`. Current cargo-deb defaults to xz. Ubuntu 14.04 and Debian 8 dpkg understand gzip. The package metadata is `[package.metadata.deb]` in `Cargo.toml`: maintainer, section, the files to install, and an empty `depends` so the static binary does not require `libc6`.

Debian names:

| Rust target | `.deb` architecture |
|---|---|
| `x86_64-unknown-linux-musl` | amd64 |
| `aarch64-unknown-linux-musl` | arm64 |
| `armv7-unknown-linux-musleabihf` | armhf |
| `i686-unknown-linux-musl` | i386 |

`cargo-deb` is a packaging tool. It is not linked into `rmbin`. The workflow installs version 3.6.0 with `--no-default-features` so that tool does not compile liblzma, and so it still builds with Rust 1.85. Gzip compression uses the Rust gzip code in cargo-deb.

There are no maintainer scripts. Installing the package copies `/usr/bin/rmbin`. Removing it deletes that file only.

### 4. Mac builds, optional

The main Release workflow does not build macOS. Both Mac runner pools were blocking pull requests: the arm64 pool never handed out a machine, and the Intel job was cancelled while queued because a newer CI run was waiting on the same concurrency group. That cancellation is not a code failure.

Two optional workflows build the Mac binaries. Neither runs on a pull request, so neither is a required check.

| Workflow | When it runs | What it uploads on a tag |
|---|---|---|
| [release-apple-silicon.yml](../.github/workflows/release-apple-silicon.yml) | Actions → Run workflow, or a `v*` tag | `rmbin-vX.Y.Z-aarch64-apple-darwin.tar.gz` and a `.sha256` file |
| [release-macos-intel.yml](../.github/workflows/release-macos-intel.yml) | Actions → Run workflow, or a `v*` tag | `rmbin-vX.Y.Z-x86_64-apple-darwin.tar.gz` and a `.sha256` file |

Each has its own concurrency group and `cancel-in-progress: false`. The main CI cannot cancel them. `gh release upload --clobber` adds the files to the release the main workflow created, or creates that release if it is not there yet.

### 5. Universal Mac binary, also optional

After both tarballs above are on the release, run [release-macos-universal.yml](../.github/workflows/release-macos-universal.yml) by hand. Type the tag, for example `v0.0.1`. It downloads the two archives onto an Intel Mac runner, runs `lipo -create`, and uploads `rmbin-universal-apple-darwin.tar.gz` plus its `.sha256`. It does not run on a tag by itself, because the two Mac jobs may still be queued. It is a CLI binary, not an `.app`. Homebrew uses that file once you paste the checksum.

### 6. Checksums

One Ubuntu job downloads every archive and every `.deb`. It writes `SHA256SUMS` with `sha256sum`. That file lists the tarballs, the zip, the universal archive, and the four `.deb` files. It does not list itself.

Check a download with:

```bash
sha256sum -c SHA256SUMS
```

### 7. GitHub Release

The same job creates the GitHub Release for the tag and attaches those files plus `SHA256SUMS`. The release text starts from [RELEASE_NOTES_TEMPLATE.md](RELEASE_NOTES_TEMPLATE.md), with `VERSION` replaced by the Cargo version. The notes state the minimum systems.

If any matrix leg fails, the release is not created. A partial set of binaries is not published.

### 8. Homebrew and apt, by hand

The workflow stops at the GitHub Release. It cannot push to `pesty-sh/homebrew-tap` or `pesty-sh/apt`.

Homebrew, after the release exists: copy the `sha256` of `rmbin-universal-apple-darwin.tar.gz` from `SHA256SUMS` into `packaging/homebrew/rmbin.rb`, uncomment `url` and `sha256`, and commit that formula in the tap. `--HEAD` still builds from source. The stable install unpacks the universal binary and does not compile.

apt: on the machine that holds the signing key, pass the four `.deb` files to `publish.py` in `pesty-sh/apt`, then commit `pool/` and `dists/`. See [PUBLISHING.md](PUBLISHING.md).

## What does not happen

- No tag is created by CI.
- crates.io is not updated.
- Binaries are not signed.
- There is no build per macOS version and no `.app`.
