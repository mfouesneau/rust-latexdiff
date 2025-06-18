# Makefile for rust-latexdiff

# Default target
all: build test

# Build in debug mode
build:
	cargo build

# Build in release mode
release:
	cargo build --release

# Run tests
test:
	cargo test

# Clean build artifacts
clean:
	cargo clean

# Install locally (requires sudo)
install: release
	sudo cp ./target/release/rust-latexdiff /usr/local/bin/
	@echo "rust-latexdiff installed to /usr/local/bin/"

# Uninstall
uninstall:
	sudo rm -f /usr/local/bin/rust-latexdiff
	@echo "rust-latexdiff uninstalled"

# Run examples
examples: release
	@echo "=== Expanding main.tex ==="
	./target/release/rust-latexdiff expand examples/main.tex -o examples/expanded_output.tex
	@echo ""
	@echo "=== Generating diff ==="
	./target/release/rust-latexdiff diff examples/chapter1.tex examples/chapter1_v2.tex -o examples/diff_output.tex
	@echo ""
	@echo "Example outputs created in examples/ directory"

# Help
help:
	@echo "Available targets:"
	@echo "  build     - Build in debug mode"
	@echo "  release   - Build in release mode"
	@echo "  test      - Run tests"
	@echo "  clean     - Clean build artifacts"
	@echo "  install   - Install to /usr/local/bin (requires sudo)"
	@echo "  uninstall - Remove from /usr/local/bin (requires sudo)"
	@echo "  examples  - Run example commands"
	@echo "  help      - Show this help"

.PHONY: all build release test clean install uninstall examples help
