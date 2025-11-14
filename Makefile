.PHONY: help run test fmt clippy check commit

# Default target
help:
	@echo "Available commands:"
	@echo "  run     - Launch the program"
	@echo "  test    - Run all tests"
	@echo "  fmt     - Format code with cargo fmt"
	@echo "  clippy  - Run linter with cargo clippy"
	@echo "  check   - Run fmt, clippy and test before committing"
	@echo "  help    - Show this help message"

# Launch the program
run:
	cargo run

# Run tests
test:
	cargo test

# Format code
fmt:
	cargo fmt

# Run linter
clippy:
	cargo clippy -- -D warnings

# Combined check before committing
check: fmt clippy test
