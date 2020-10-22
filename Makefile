.PHONY: dev test doc

dev:
	rustup component add rustfmt clippy

test:
	cargo fmt --all -- --check
	cargo clippy --all --all-targets --all-features -- -D warnings
	cargo test --all --locked

publish:
	cargo login "${CARGO_LOGIN_TOKEN}"
	cargo publish
