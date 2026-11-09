# Publishing 0.0.1

Nothing in this repository publishes a release on its own. The owner does these steps after reviewing the pull request. This environment cannot push to `pesty-sh/homebrew-tap` or `pesty-sh/apt` (no write permission). The formula and the `.deb` packer are in this repo so they can be copied by hand.

## Before the tag

1. Merge the pull request to `main`.
2. Confirm the licence. It is provisionally MIT OR Apache-2.0. If that is wrong, change `LICENSE`, `Cargo.toml`, and the Homebrew formula before tagging.
3. Re-read [COMPARISON.md](COMPARISON.md). It is a draft with placeholders. Do not publish it as a claim. Fill the contact line only when you want that page public, and only after the checklist inside the file is done.
4. Run the manual rows in [TESTING.md](TESTING.md) that you care about for this alpha. CI does not include a macOS 13 machine.
5. Check the name `rmbin` is still free on crates.io if you plan to publish there. The roadmap noted a 404 on 2026-11-09. Check again.

## GitHub Release

The workflow [`.github/workflows/release.yml`](../.github/workflows/release.yml) starts when a tag matching `v*` is pushed. Pushing the tag **creates the GitHub Release**. Do not push it until you want that.

```bash
git checkout main
git pull origin main
git tag -a v0.0.1 -m "rmbin 0.0.1"
git push origin v0.0.1
```

Wait until the Release workflow is green. The release should contain:

- `rmbin-v0.0.1-x86_64-apple-darwin.tar.gz`
- `rmbin-v0.0.1-aarch64-apple-darwin.tar.gz`
- `rmbin-v0.0.1-x86_64-unknown-linux-musl.tar.gz`
- `rmbin-v0.0.1-aarch64-unknown-linux-musl.tar.gz`
- `rmbin-v0.0.1-x86_64-pc-windows-msvc.zip`
- `rmbin_0.0.1-1_amd64.deb` and `rmbin_0.0.1-1_arm64.deb`
- `SHA256SUMS`

Verify on a second machine:

```bash
sha256sum -c SHA256SUMS
```

Archives are not signed. Gatekeeper and SmartScreen can warn. Signing keys are still an open decision. A Homebrew install compiles from source and does not download the binary archive.

The tag must be `v0.0.1` because the workflow checks it against `version` in `Cargo.toml`.

## Homebrew

Trust is per tap, once per Mac:

```bash
brew tap pesty-sh/tap
brew trust --tap pesty-sh/tap
```

Copy [`packaging/homebrew/rmbin.rb`](../packaging/homebrew/rmbin.rb) to `Formula/rmbin.rb` in [pesty-sh/homebrew-tap](https://github.com/pesty-sh/homebrew-tap). Until the tag exists, the formula is head-only:

```bash
HOMEBREW_NO_AUTO_UPDATE=1 brew install --HEAD pesty-sh/tap/rmbin
```

After `v0.0.1` is on GitHub, checksum **GitHub's** tarball, not a local `git archive`:

```bash
curl -fsSL -o /tmp/rmbin.tar.gz https://github.com/pesty-sh/rmbin/archive/refs/tags/v0.0.1.tar.gz
shasum -a 256 /tmp/rmbin.tar.gz
```

Paste that hash into `sha256`, uncomment `url` and `sha256` in the tap's formula, commit, and push the tap. Then:

```bash
brew update
brew install pesty-sh/tap/rmbin
```

`cargo install --locked` is what Homebrew's Rust helper runs when `Cargo.lock` is present. The formula does not delete user files.

## apt

The Linux archives are static musl binaries packed by [`packaging/mkdeb.py`](../packaging/mkdeb.py). They do not depend on a particular glibc. There are no maintainer scripts.

On the machine that holds the apt signing key for [pesty-sh/apt](https://github.com/pesty-sh/apt), pass the two `.deb` files from the GitHub Release to that repo's `publish.py`, then commit `pool/` and `dists/` the same way qrbin is published. The fingerprint and the `signed-by` line already used by that repo stay as they are.

An optional pin file is [`packaging/apt/rmbin.pref`](../packaging/apt/rmbin.pref). It is not required when Debian and Ubuntu do not ship a package of the same name.

Installing or removing the `.deb` only adds or removes `/usr/bin/rmbin`.

## crates.io

```bash
cargo publish --locked
```

Do this only after the licence is confirmed and the version is the one you want public. The GitHub Release, the tap, apt, and crates.io should all say 0.0.1.
