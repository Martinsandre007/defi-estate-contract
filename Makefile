# Run the same checks as CI before you push.

.PHONY: check fix build test help

help:
	@echo "make check   fmt, clippy, test and wasm build (same as CI)"
	@echo "make fix     cargo fmt"
	@echo "make build   build optimized wasm"
	@echo "make test    cargo test"

check:
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -- -D warnings
	cargo test
	cargo build --target wasm32v1-none --release

fix:
	cargo fmt --all

build:
	./scripts/build-contracts.sh

test:
	cargo test
</br>