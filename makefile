# Make will use bash instead of sh
SHELL := /usr/bin/env bash

.PHONY: help
help:
	@echo ' '
	@echo '    make build   	Builds the code base incrementally (fast) for dev.'
	@echo '    make bench   	Runs all benchmarks across all crates.'
	@echo '    make check   	Checks the code base for security vulnerabilities.'
	@echo '    make check_examples	Checks that every Cargo example has a Bazel target.'
	@echo '    make fix   		Fixes linting issues as reported by clippy.'
	@echo '    make format   	Formats call code according to cargo fmt style.'
	@echo '    make install   	Tests and installs all make script dependencies.'
	@echo '    make lean       	Runs all LEAN proofs'
	@echo '    make miri   	Runs the test suite under the Miri interpreter for undefined-behavior detection.'
	@echo '    make start   	Starts the dev day with updating rust, pulling from git remote, and build the project.'
	@echo '    make test   	Runs all tests across all crates.'
	@echo '    make update   	Update all dependencies for all crates'
	@echo '    make sbom   	Generate SBOM for all crates'

# "---------------------------------------------------------"
# "---------------------------------------------------------"

.PHONY: build
build:
	@source scripts/build.sh


.PHONY: bench
bench:
	@source scripts/bench.sh


.PHONY: check
check:
	@source scripts/check.sh


.PHONY: check_examples
check_examples:
	@source scripts/check_examples.sh


.PHONY: fix
fix:
	@source scripts/fix.sh


.PHONY: format
format:
	@source scripts/format.sh


.PHONY: install
install:
	@source scripts/install_deps.sh


.PHONY: lean
lean:
	@source scripts/lean.sh


.PHONY: miri
miri:
	@source scripts/miri.sh


.PHONY: release
release:
	@source scripts/release.sh


.PHONY: start
start:
	@source scripts/start.sh


.PHONY: test
test:
	@source scripts/test.sh


.PHONY: sbom
sbom:
	 @source scripts/sbom.sh


.PHONY: update
update:
	 @source scripts/update.sh
