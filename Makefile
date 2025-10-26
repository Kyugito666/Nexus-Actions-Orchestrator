# Makefile

.PHONY: build run clean install test help

help:
	@echo "Nexus GitHub Orchestrator - Build Commands"
	@echo ""
	@echo "make build    - Build release binary"
	@echo "make run      - Run orchestrator"
	@echo "make test     - Run tests"
	@echo "make install  - Install dependencies"
	@echo "make clean    - Clean build artifacts"

build:
	cargo build --release

run:
	cargo run --release

test:
	cargo test

clean:
	cargo clean
	rm -rf target/
	rm -rf logs/*.log

install:
	@echo "Installing dependencies..."
	@if command -v apt-get >/dev/null 2>&1; then \
		echo "Installing libsodium-dev..."; \
		sudo apt-get update && sudo apt-get install -y libsodium-dev; \
	elif command -v brew >/dev/null 2>&1; then \
		echo "Installing libsodium..."; \
		brew install libsodium; \
	else \
		echo "Please install libsodium manually"; \
		exit 1; \
	fi
	@echo "Building project..."
	cargo build --release
	@echo "Done!"

status:
	cargo run --release -- status

billing:
	cargo run --release -- billing

cleanup:
	cargo run --release -- cleanup
