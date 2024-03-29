.PHONY: check
check:
	@cargo check

.PHONY: build
build:
	@cargo build

.PHONY: test
test:
	@cargo test

.PHONY:
cargo-publish:
	@cargo publish --dry-run

setup:
	@rustup default stable
	@rustup component add rls rust-analysis rust-src
