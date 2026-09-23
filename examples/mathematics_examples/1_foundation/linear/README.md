# Foundation: `deep_causality_linear`

Sparse and dense linear algebra. `CsrMatrix` stores only its explicit entries, so memory
scales with their count.

| Example | What it covers | Command |
|---|---|---|
| [basic_csr_ops.rs](basic_csr_ops.rs) | Construction from triplets, scalar scaling, addition, matrix-vector and matrix-matrix products, transpose, and the error a dimension mismatch returns | `cargo run -p mathematics_examples --example basic_csr_ops_examples` |
| [dense_matrix_stencil.rs](dense_matrix_stencil.rs) | `DenseMatrixWitness` as a comonad: `extend` focusing each entry at `(0, 0)` on a wrapped grid, a nine-point filter written once with no border case, and the sum the filter conserves | `cargo run -p mathematics_examples --example dense_matrix_stencil_examples` |

`CsrMatrixWitness` carries `Functor`, `Pure`, `Applicative`, `CoMonad` and `Foldable`; those are
shown in `2_composition/extension/sparse_matrix_contextual_sum/`.
