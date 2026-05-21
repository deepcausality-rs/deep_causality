# CausalTensor Examples

Examples for the `deep_causality_tensor` crate. `CausalTensor<T>` is the N-dimensional
tensor type that underpins field-on-cells storage, Einstein-summation contractions,
and the HKT trait surface (`Functor`, `Applicative`, `Monad`).

Run from the repository root:

```bash
cargo run -p mathematics_examples --example <example_name>
```

| File | Description | Command |
|------|-------------|---------|
| [basic_causal_tensor.rs](basic_causal_tensor.rs) | Construction (`new`, `from_vec`), shape and slice access, element-wise ops. Start here | `cargo run -p mathematics_examples --example basic_causal_tensor_examples` |
| [functor_causal_tensor.rs](functor_causal_tensor.rs) | `Functor::fmap` over a `CausalTensor` via `CausalTensorWitness`; cross-precision mapping (`f64 → f32`) | `cargo run -p mathematics_examples --example functor_causal_tensor_examples` |
| [applicative_causal_tensor.rs](applicative_causal_tensor.rs) | `Applicative::apply` applies a tensor of functions to a tensor of values element-wise | `cargo run -p mathematics_examples --example applicative_causal_tensor_examples` |
| [ein_sum_causal_tensor.rs](ein_sum_causal_tensor.rs) | Einstein-summation contractions: matrix multiplication, batched contractions, partial reductions via `EinSumOp` | `cargo run -p mathematics_examples --example ein_sum_causal_tensor_examples` |
| [einstein_field_causal_tensor.rs](einstein_field_causal_tensor.rs) | Tensor algebra for general relativity: index raising/lowering with the metric, Ricci-style contractions | `cargo run -p mathematics_examples --example einstein_field_causal_tensor_examples` |
| [effect_system_causal_tensor.rs](effect_system_causal_tensor.rs) | `CausalTensor` inside the type-encoded effect system; tracking errors and logs through tensor pipelines | `cargo run -p mathematics_examples --example effect_system_causal_tensor_examples` |
