# Foundation: `deep_causality_tensor`

N-index tensors: flat data plus a shape, with broadcasting, axis reductions and Einstein
summation over them.

| Example | What it covers | Command |
|---|---|---|
| [basic_causal_tensor.rs](basic_causal_tensor.rs) | Construction, indexing, reshape and ravel, scalar and tensor arithmetic with broadcasting, `sum_axes` / `mean_axes`, `arg_sort`, element-wise logarithms, and `stack` | `cargo run -p mathematics_examples --example basic_causal_tensor_examples` |
| [ein_sum_causal_tensor.rs](ein_sum_causal_tensor.rs) | `EinSumOp` as an expression value: `mat_mul`, `dot_prod`, `trace`, `element_wise_product` and `batch_mat_mul`, each checked against the hand-worked result | `cargo run -p mathematics_examples --example ein_sum_causal_tensor_examples` |
| [tensor_train_compression.rs](tensor_train_compression.rs) | `CausalTensorTrainWitness`: a tensor stored as its cores, the bond dimensions the TT-SVD finds, and the fact that `fmap` and `fold` act on the factors rather than the tensor they represent | `cargo run -p mathematics_examples --example tensor_train_compression_examples` |

`CausalTensorWitness` implements `Monad`, `CoMonad`, `Traversable`, `DiagonalTraversable` and
`Collectable`, among others; `2_composition/` exercises them.
