# QCL model path: the shipped freeze checks through the pipeline

The `qcm_freeze_check` example runs the Markov commutativity check at the freeze boundary. This
example runs the same checks through the QCL pipeline and shows that the two agree.

```bash
cargo run -p quantum_examples --example qcl_qcm_freeze
```

Three scenarios:

1. A commuting model screens. `validate` runs `check_markov` and `check_decomposable`, the two
   level checks `freeze_quantum` runs inside `freeze_verified_with_check`, and terminates in a
   `Screened` whose report names the pairs examined and the worst margin. The shipped freeze on
   the same model reports the same pair count and margin.
2. A non-commuting model fails `validate` with `CommutatorNonZero` naming the pair. The frozen
   subject is left exactly as built, because the pipeline mutates nothing.
3. The same model on a dynamic graph through `QclBuilder::freeze_model`, the shipped freeze,
   which rolls the graph back to its dynamic state.

The decomposability check on a 1 × 1 system relation examines no 3 × 3 block and reads as a
vacuous pass, which the report says rather than hides.
