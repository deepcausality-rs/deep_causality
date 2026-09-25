# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_quantum-v0.3.0...deep_causality_quantum-v0.4.0) - 2026-09-23

### Added

- *(quantum)* [**breaking**] let ? join a multivector construction to a kernel, and print laws readably
- *(deep_causality_quantum)* the crosstalk decision over circuit-derived candidates, and the QCL-2 close-out (group 8)
- *(deep_causality_quantum)* the decoder as an abstraction over a detector error model (QCL-2 group 7)
- *(deep_causality_quantum)* abstraction composition, its law with two computed constants, and three chains (QCL-2 group 6)

### Fixed

- *(deep_causality_quantum)* resolve the QCL-2 review findings on composition, norms, DEM caps and the specifications

### Other

- edit all READMEs for clarity, concision and correctness
- *(deep_causality_quantum)* regenerated SBOM.
- *(examples)* fold chronometric_examples into physics_examples
- Update deep_causality_quantum/tests/types/abstraction/composition_tests.rs
- Update deep_causality_quantum/src/types/abstraction/chains.rs

## [0.3.0](https://github.com/deepcausality-rs/deep_causality/compare/deep_causality_quantum-v0.2.5...deep_causality_quantum-v0.3.0) - 2026-09-15

### Added

- *(deep_causality_quantum)* Lining, fixes, and other improvements.
- *(deep_causality_quantum)* fault sets, the fault-tolerance predicate and the Haruna filter (QCL-2 group 5)
- *(deep_causality_quantum)* structural precheck, code abstraction and the abstraction pipeline stages (QCL-2 group 4)
- *(deep_causality_quantum)* type alignment, query signatures, abstractions and the naturality check (QCL-2 group 3)
- *(deep_causality_quantum)* Add the circuit model, its two semantics, and the BLO dilation (QCL-2 groups 0-2)

### Fixed

- *(deep_causality_quantum)* widen the chain-complex check and accept repeated interchange nodes
- *(deep_causality_quantum)* resolve the QCL-2 review findings on caps, ordering, validation and errors

### Other

- *(uncertain)* [**breaking**] migrate the consumers, and take the last lifetime off the scalar
- *(uncertain)* [**breaking**] replace the global sample cache with a caller-owned session
- *(deep_causality_quantum)* close the coverage gaps in the qpu, circuit and pipeline layers
- *(openspec)* Updated specs for quantum crate.
