# Foundation: `deep_causality_tensor`

N-index tensors: flat data plus a shape, with broadcasting, axis reductions and Einstein
summation over them.

| Example | What it covers | Command |
|---|---|---|
| [basic_causal_tensor.rs](basic_causal_tensor.rs) | Construction, indexing, reshape and ravel, scalar and tensor arithmetic with broadcasting, `sum_axes` / `mean_axes`, `arg_sort`, element-wise logarithms, and `stack` | `cargo run -p mathematics_examples --example basic_causal_tensor_examples` |
| [ein_sum_causal_tensor.rs](ein_sum_causal_tensor.rs) | `EinSumOp` as an expression value: `mat_mul`, `dot_prod`, `trace`, `element_wise_product` and `batch_mat_mul`, each checked against the hand-worked result | `cargo run -p mathematics_examples --example ein_sum_causal_tensor_examples` |

`CausalTensorWitness` carries the widest trait set in the workspace — `Monad`, `CoMonad`,
`Traversable`, `DiagonalTraversable` and `Collectable` among them — exercised across
`2_composition/`.
