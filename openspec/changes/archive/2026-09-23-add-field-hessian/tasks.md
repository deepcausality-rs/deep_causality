Follows the `unified-math-tdd-protocol`. The record is `tdd-notes.md` beside this file.

## 1. Field Hessian

- [x] 1.1 Phase 1: declare `DifferentiateFieldExt::hessian` with an `unimplemented!()` body; build
      under Cargo and Bazel
- [x] 1.2 Phase 2: enumerate corner cases; write the suite in
      `tests/extensions/differentiate_ext_tests.rs`; observe every test fail on the unimplemented panic
- [x] 1.3 Phase 3: audit the suite against 14 deliberate defects in a throwaway implementation
- [x] 1.4 Phase 4: implement; clippy clean; Cargo and Bazel counts agree; full coverage of the file
- [x] 1.5 Phase 5: `scripts/mutants.sh deep_causality_calculus src/extensions/differentiate_ext.rs`
- [x] 1.6 Document `hessian` in the crate README and the crate-level rustdoc
