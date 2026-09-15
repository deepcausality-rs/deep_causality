# Foundation: `deep_causality_linear`

Sparse and dense linear algebra. `CsrMatrix` stores the entries that exist, so memory tracks
the stored entries.

| Example | What it covers | Command |
|---|---|---|
| [basic_csr_ops.rs](basic_csr_ops.rs) | Construction from triplets, scalar scaling, addition, matrix-vector and matrix-matrix products, transpose, and the error a dimension mismatch returns | `cargo run -p mathematics_examples --example basic_csr_ops_examples` |

`CsrMatrixWitness` carries `Functor`, `Pure`, `Applicative`, `CoMonad` and `Foldable`; those are
shown in `2_composition/extension/sparse_matrix_contextual_sum/`.
