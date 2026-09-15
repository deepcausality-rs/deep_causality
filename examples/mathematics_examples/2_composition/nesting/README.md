# Nesting

A witness accepts any element type, including one another crate owns. The container's `Functor`
walks the outer structure; the element's own algebra does the work at each slot. Nothing is
converted, because the element type never left its crate.

| Example | What it shows | Command |
|---|---|---|
| [tensor_x_algebra_rotation_field](tensor_x_algebra_rotation_field/) | A `CausalTensor<CausalMultiVector<T>>` is a discrete vector field. One `CausalTensorWitness::fmap` rotates every cell by a single `Cl(2,0)` rotor, `R v R~` | `cargo run -p mathematics_examples --example tensor_x_algebra_rotation_field_examples` |
