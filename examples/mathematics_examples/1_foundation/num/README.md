# Foundation: the number tower

`deep_causality_num` and `deep_causality_num_complex`. The scalars everything else is written
over, the crossings into and out of them, and the two algebras that are not approximations of the
reals.

Run from the repository root:

```bash
cargo run -p mathematics_examples --example <example_name>
```

| File | What it covers | Command |
|---|---|---|
| [scalars_and_precision.rs](scalars_and_precision.rs) | The four types that implement `Float` — `BFloat16`, `f32`, `f64`, `Float106` — with every function written once against the bound. Compensated summation against naive, and the rearrangement of `(1 − cos x)/x²` that removes the cancellation | `cargo run -p mathematics_examples --example scalars_and_precision_examples` |
| [precision_boundary.rs](precision_boundary.rs) | `lift`, `lift_count`, `lower` and `to_count`: the two places a program written against a scalar parameter meets the concrete world, what a decimal literal costs at each width, and which crossings refuse | `cargo run -p mathematics_examples --example precision_boundary_examples` |
| [binary_field_gf2.rs](binary_field_gf2.rs) | `Gf2`, the field with two elements: characteristic 2 and what follows from it, RAID-5 parity and erasure recovery, and Hamming(7,4) single-error correction | `cargo run -p mathematics_examples --example binary_field_gf2_examples` |
| [cayley_dickson_ladder.rs](cayley_dickson_ladder.rs) | `ComplexWitness`, `QuaternionWitness` and `OctonionWitness` carrying the same six traits, so one norm and one zip serve 2, 4 and 8 slots; then what each rung keeps — the multiplicative norm on all three, commutativity through ℂ, associativity through ℍ | `cargo run -p mathematics_examples --example cayley_dickson_ladder_examples` |

## The scalars

`Float` is implemented by exactly four types, spanning nine orders of magnitude in resolution:

| Type | Significand bits | Decimal digits |
|---|---|---|
| `BFloat16` | 8 | 2.1 |
| `f32` | 24 | 6.9 |
| `f64` | 53 | 15.7 |
| `Float106` | 106 | 31.3 |

`Float106` is two `f64` limbs held as an unevaluated sum, which is why it keeps `f64`'s exponent
range. It is also the oracle the narrower paths are diffed against.

## The two that are not floats

`Gf2` is a field with two elements, where addition is exclusive-or and every element is its own
additive inverse. It is exact, so no `FloatType` alias appears in its example: precision is a
parameter for scalars that approximate the reals, and there is nothing here to round.

`Quaternion` and `Octonion` extend the reals rather than approximating them differently. Octonion
multiplication is derived from the Cayley–Dickson doubling of the quaternions, which is what
carries the composition law `|xy| = |x||y|` up from the rung below.
