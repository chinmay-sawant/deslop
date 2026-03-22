.PHONY: build test clean fmt lint help scan

# Path to scan (can be overridden with 'make scan PATH=<path>')
# For now hardcoded the path for personal use, you can customize this as required
PATH_TO_SCAN ?= /home/chinmay/ChinmayPersonalProjects/gopdfsuit
# PATH_TO_SCAN ?= .

# Default Target
all: help

# Build the project
build:
	cargo build

# Run project tests
test:
	cargo test

# Check code for linting issues using clippy
lint:
	cargo clippy --all-targets --all-features

# Apply code formatting
fmt:
	cargo fmt

# Run the analyzer on the target path
scan:
	cargo run -- scan $(PATH_TO_SCAN)

# Clean build artifacts and temporary files
clean:
	cargo clean
	rm -f results.txt

# Display help for make targets
help:
	@echo "Deslop Makefile Targets:"
	@echo "  build       - Compile the binary"
	@echo "  test        - Run the test suite"
	@echo "  lint        - Run clippy for code analysis"
	@echo "  fmt         - Format source code"
	@echo "  scan        - Run deslop scan on a directory (default PATH_TO_SCAN=.)"
	@echo "                Example: make scan PATH_TO_SCAN=/path/to/go/project"
	@echo "  clean       - Remove build artifacts and result files"
	@echo "  help        - Show this menu"