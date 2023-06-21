# Make will use bash instead of sh
SHELL := /usr/bin/env bash
# GNU make man page: http://www.gnu.org/software/make/manual/make.html

.PHONY: help
help:
	@echo ' '
	@echo '    make build   	Builds the code base incrementally (fast) for dev.'
	@echo '    make bench   	Runs all benchmarks across all crates.'
	@echo '    make check   	Checks the code base for security vulnerabilities.'
	@echo '    make example   	Runs the default example: Smoking.'
	@echo '    make fix   		Fixes linting issues as reported by cargo'
	@echo '    make test   		Runs all tests in the target defined.'

# "---------------------------------------------------------"
# Development
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


.PHONY: clean
clean:
	@source scripts/clean.sh


PHONY: example
example:
	@source scripts/example.sh


.PHONY: fix
fix:
	@source scripts/fix.sh


.PHONY: test
test:
	@source scripts/test.sh
