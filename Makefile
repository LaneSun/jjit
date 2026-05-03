.PHONY: all build install uninstall test clean help

# Default target
all: build

# Build release binary
build:
	@echo "Building jjit..."
	@cargo build --release

# Install to ~/.local/bin or $INSTALL_DIR
install: build
	@echo "Installing jjit..."
	@./install.sh

# Uninstall
uninstall:
	@./install.sh uninstall

# Run tests
test:
	@cargo test

# Clean build artifacts
clean:
	@cargo clean
	@rm -rf target/

# Format code
fmt:
	@cargo fmt

# Run clippy
lint:
	@cargo clippy -- -D warnings

# Check code
check:
	@cargo check

# Show help
help:
	@echo "Available targets:"
	@echo "  build      - Build release binary"
	@echo "  install    - Install jjit to system"
	@echo "  uninstall  - Remove jjit from system"
	@echo "  test       - Run tests"
	@echo "  clean      - Clean build artifacts"
	@echo "  fmt        - Format code"
	@echo "  lint       - Run clippy lints"
	@echo "  check      - Check code"
	@echo "  help       - Show this help"
