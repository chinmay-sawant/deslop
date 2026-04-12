.PHONY: build test clean fmt lint help scan scan-info scan-gopdfsuit scan-gopdfsuit-info scan-snapback scan-snapback-info scan-claw scan-claw-info

# Path to scan (can be overridden with 'make scan PATH=<path>')
# For now hardcoded the path for personal use, you can customize this as required
PATH_TO_SCAN_GOPDFSUIT ?= /home/chinmay/ChinmayPersonalProjects/gopdfsuit
PATH_TO_SCAN_SNAPBACK ?= /home/chinmay/ChinmayPersonalProjects/SnapBack
PATH_TO_SCAN ?= .
PATH_TO_SCAN_CLAW ?= /home/chinmay/ChinmayPersonalProjects/deslop/real-repos/claw-code-main
WSL_REPO_ROOT ?= /home/chinmay/ChinmayPersonalProjects/deslop-codex
CARGO ?= cargo

ifeq ($(OS),Windows_NT)
ifeq ($(shell where cargo 2>NUL),)
CARGO := C:\\Windows\\Sysnative\\wsl.exe --cd $(WSL_REPO_ROOT) cargo
endif
endif

# Default Target
all: help

# Build the project
build:
	$(CARGO) build

# Run project tests
test:
	$(CARGO) test

# Check code for linting issues using clippy
lint:
	$(CARGO) clippy --all-targets --all-features
	$(CARGO) fmt

# Apply code formatting
fmt:
	$(CARGO) fmt

# Run the analyzer on the target path
# deslop exits with a non-zero status code (1) whenever it detects an issue
scan:
	$(CARGO) run -- scan $(PATH_TO_SCAN) > results.txt
	python3 scripts/extract_finding_context.py results.txt

scan-info:
	$(CARGO) run -- scan $(PATH_TO_SCAN) --no-fail > results.txt
	python3 scripts/extract_finding_context.py results.txt

scan-gopdfsuit:
	$(CARGO) run -- scan $(PATH_TO_SCAN_GOPDFSUIT) > temp_gopdfsuit.txt

scan-gopdfsuit-info:
	$(CARGO) run -- scan $(PATH_TO_SCAN_GOPDFSUIT) --no-fail > temp_gopdfsuit.txt
	python3 scripts/extract_finding_context.py temp_gopdfsuit.txt

scan-snapback:
	$(CARGO) run -- scan $(PATH_TO_SCAN_SNAPBACK) > temp_snapback.txt

scan-snapback-info:
	$(CARGO) run -- scan $(PATH_TO_SCAN_SNAPBACK) --no-fail > temp_snapback.txt
	python3 scripts/extract_finding_context.py temp_snapback.txt

scan-claw:
	$(CARGO) run -- scan $(PATH_TO_SCAN_CLAW) --ignore hallucinated_import_call > temp_claw.txt

scan-claw-info:
	$(CARGO) run -- scan $(PATH_TO_SCAN_CLAW) --ignore hallucinated_import_call --no-fail > temp_claw.txt
	python3 scripts/extract_finding_context.py temp_claw.txt

# Clean build artifacts and temporary files
clean:
	$(CARGO) clean
	rm -f results.txt temp_gopdfsuit.txt temp_snapback.txt temp_claw.txt

temp:
	cargo run -- scan /home/chinmay/ChinmayPersonalProjects/deslop/real-repos/GO_FHIR_DEMO --no-fail > temp.txt
	python3 scripts/extract_finding_context.py temp.txt

# Display help for make targets
help:
	@echo "Deslop Makefile Targets:"
	@echo "  build       - Compile the binary"
	@echo "  test        - Run the test suite"
	@echo "  lint        - Run clippy for code analysis"
	@echo "  fmt         - Format source code"
	@echo "  scan        - Run deslop scan on a directory (default PATH_TO_SCAN=.)"
	@echo "  scan-info   - Run deslop scan without failing make when findings are present"
	@echo "                Example: make scan PATH_TO_SCAN=/path/to/go/project"
	@echo "  clean       - Remove build artifacts and result files"
	@echo "  help        - Show this menu"
	@echo "  scan-gopdfsuit - Scan the gopdfsuit project and save results to temp_gopdfsuit.txt"
	@echo "  scan-gopdfsuit-info - Informational gopdfsuit scan that keeps output but does not fail make"
	@echo "  scan-snapback - Scan the snapback project and save results to temp_snapback.txt"
	@echo "  scan-snapback-info - Informational snapback scan that keeps output but does not fail make"
	@echo "  scan-claw - Scan the claw project and save results to temp_claw.txt"
	@echo "  scan-claw-info - Informational claw scan that keeps output but does not fail make"
	@echo "  temp        - Run a temporary scan on the real repo project and process results"
