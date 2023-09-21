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
	@echo '    make test   	Runs all tests across all crates.'

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


.PHONY: example
example:
	@source scripts/example.sh


.PHONY: fix
fix:
	@source scripts/fix.sh


.PHONY: format
format:
	@source scripts/format.sh


.PHONY: install
install:
	@source scripts/install_deps.sh


.PHONY: start
start:
	@source scripts/start.sh


.PHONY: test
test:
	@source scripts/test.sh
