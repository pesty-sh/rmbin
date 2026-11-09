# Build rmbin for this machine. The binary stays in target/.
# It is not copied onto PATH, and it does not download anything when it runs.
# See docs/OFFLINE.md for a fully offline build.

.PHONY: build test check

build:
	cargo build --release --locked
	@echo "target/release/rmbin"

test:
	cargo test --locked

check: test
	cargo fmt --all -- --check
	cargo clippy --all-targets --locked -- -D warnings
	./packaging/check-no-network.sh
	./packaging/check-no-c.sh
	./packaging/check-linked-libs.sh target/debug/rmbin
