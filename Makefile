.PHONY: build dev test test-rust test-python lint typecheck check clean fmt wasm

# Build the Rust extension (release mode)
build:
	uv run maturin develop --release

# Build the Rust extension (debug mode, faster compile)
dev:
	uv run maturin develop

# Install all dependencies
install:
	uv sync

# Run all tests (Rust + Python)
test: test-rust test-python

# Rust unit tests (core crate only — no PyO3 in test runner)
test-rust:
	cargo test -p darkhex-core

# Python integration tests (rebuilds Rust extension first)
test-python: dev
	uv run pytest -x

# Lint Python code
lint:
	uv run ruff check .

# Lint and auto-fix Python code
lint-fix:
	uv run ruff check --fix .

# Type check Python code
typecheck:
	uv run pyright

# Format Rust code
fmt:
	cargo fmt --all

# Build WASM package for DSaGe web app
wasm:
	cd crates/wasm && wasm-pack build --target web --out-dir ../../game/pkg

# Full verification (lint + test)
check:
	./scripts/run-check.sh uv run ruff check .
	./scripts/run-check.sh cargo test -p darkhex-core
	./scripts/run-check.sh uv run pytest -x

# Clean build artifacts
clean:
	cargo clean
	rm -rf .pytest_cache __pycache__ darkhex/__pycache__
	find . -name "*.so" -delete
	find . -name "*.dylib" -delete
	find . -name "*.pyd" -delete
