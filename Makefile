.PHONY: help build release run test fmt clippy install command-generator-build

help:
	@echo "Targets:"
	@echo "  make build                   Build debug binary"
	@echo "  make release                 Build release binary"
	@echo "  make run ARGS='...'          Run with optional args"
	@echo "  make test                    Run test suite"
	@echo "  make fmt                     Run rustfmt"
	@echo "  make clippy                  Run clippy with warnings as errors"
	@echo "  make install                 Install binary from local path"
	@echo "  make command-generator-build Run helper build tool"

build:
	cargo build

release:
	cargo build --release

run:
	cargo run -- $(ARGS)

test:
	cargo test

fmt:
	cargo fmt

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

install:
	cargo install --path .

command-generator-build:
	cargo run --manifest-path build/Cargo.toml --
