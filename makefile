# Make will use bash instead of sh
SHELL := /usr/bin/env bash

.PHONY: help
help:
	@echo ' '
	@echo '    make build   	Builds the code base incrementally (fast) for dev.'
	@echo '    make bench   	Runs all benchmarks across all crates.'
	@echo '    make check   	Checks the code base for security vulnerabilities.'
	@echo '    make example   	Runs the example code.'
	@echo '    make fix   		Fixes linting issues as reported by clippy.'
	@echo '    make format   	Formats call code according to cargo fmt style.'
	@echo '    make install   	Tests and installs all make script dependencies.'
	@echo '    make start   	Starts the dev day with updating rust, pulling from git remote, and build the project.'
	@echo '    make sbom   	    Generates a SBOM for each crate of the project'
	@echo '    make test   	Runs all tests across all crates.'

# "---------------------------------------------------------"
# "---------------------------------------------------------"

.PHONY: build
build:
	@source build/scripts/build.sh


.PHONY: bench
bench:
	@source build/scripts/bench.sh


.PHONY: check
check:
	@source build/scripts/check.sh


.PHONY: example
example:
	@source build/scripts/example.sh


.PHONY: fix
fix:
	@source build/scripts/fix.sh


.PHONY: format
format:
	@source build/scripts/format.sh


.PHONY: install
install:
	@source build/scripts/install_deps.sh


.PHONY: release
release:
	@source build/scripts/release.sh

.PHONY: start
start:
	@source build/scripts/start.sh


.PHONY: test
test:
	@source build/scripts/test.sh

.PHONY: sbom
sbom:
	 @source build/scripts/sbom.sh

.PHONY: vendor
vendor:
	@source build/scripts/vendor.sh