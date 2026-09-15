# Foundation: `deep_causality_multivector`

Geometric algebra. A `CausalMultiVector` holds the `2^n` blade coefficients of one point in
`Cl(p, q, r)`; a `CausalMultiField` holds one multivector per grid cell.

The storage is worth knowing up front. A field holds the **matrix isomorphism** of each
multivector, shaped `[Nx, Ny, Nz, D, D]` where `D = matrix_dim(n)`, which turns a field product
into one batched matrix multiply. Blades come back out through `to_coefficients`.

## The type

| Example | What it covers | Command |
|---|---|---|
| [basic_multivector.rs](basic_multivector.rs) | The three products — geometric, outer, inner — plus squaring, the inverse, and non-commutativity | `cargo run -p mathematics_examples --example basic_multivector_examples` |
| [matrix_representation.rs](matrix_representation.rs) | `to_matrix` / `from_matrix` / `get_gamma_matrix`, and the homomorphism that makes the representation useful: the geometric product *is* the matrix product | `cargo run -p mathematics_examples --example matrix_representation_examples` |

## The field

| Example | What it covers | Command |
|---|---|---|
| [causal_multi_field_basics.rs](causal_multi_field_basics.rs) | Construction, the `[Nx,Ny,Nz,D,D]` storage layout, grade projection, the four products, and the round trip back to individual multivectors | `cargo run -p mathematics_examples --example causal_multi_field_basics_examples` |
| [multi_field_differential.rs](multi_field_differential.rs) | The geometric derivative: `partial_derivative`, `gradient`, `divergence` and `curl`, every result checked against its closed form | `cargo run -p mathematics_examples --example multi_field_differential_examples` |

## Applied

| Example | What it covers | Command |
|---|---|---|
| [clifford_mhd_multivector.rs](clifford_mhd_multivector.rs) | Metric agnosticism: the same `F = J . B` gives the right Lorentz force in both a Euclidean and a Minkowski signature | `cargo run -p mathematics_examples --example clifford_mhd_multivector_examples` |
| [pga3d_multivector.rs](pga3d_multivector.rs) | Projective geometric algebra: points as dual tri-vectors, a translator motor, and the sandwich product that applies it | `cargo run -p mathematics_examples --example pga3d_multivector_examples` |
| [algebraic_scanner](algebraic_scanner/) | Scanning `Cl(p, q)` for the signatures whose pseudoscalar squares to `-1`, which is what lets an algebra stand in for the complex numbers | `cargo run -p mathematics_examples --example algebraic_scanner_examples` |
| [multifield_witness.rs](multifield_witness.rs) | `CausalMultiFieldWitness`: `fmap` carrying the metric, spacing and shape across, the comonad law `extend(extract) == id`, and `fmap` commuting with `gradient` | `cargo run -p mathematics_examples --example multifield_witness_examples` |

`PGA3DMultiVector` is an alias for `CausalMultiVector<f64>` and its constructors take `f64`
directly, so that one type fixes its precision at the crate.
