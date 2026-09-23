# Foundation: the number tower

Examples for `deep_causality_num` and `deep_causality_num_complex`: the scalars every other
crate is written over, the crossings into and out of them, and two algebras that do not
approximate the reals.

Run from the repository root:

```bash
cargo run -p mathematics_examples --example <example_name>
```

| File | What it covers | Command |
|---|---|---|
| [scalars_and_precision.rs](scalars_and_precision.rs) | The four types that implement `Float` (`BFloat16`, `f32`, `f64`, `Float106`), with every function written once against the bound. Compensated summation against naive, and the rearrangement of `(1 − cos x)/x²` that removes the cancellation | `cargo run -p mathematics_examples --example scalars_and_precision_examples` |
| [precision_boundary.rs](precision_boundary.rs) | `lift`, `lift_count`, `lower` and `to_count`: the two places a program written against a scalar parameter meets the concrete world, what a decimal literal costs at each width, and which crossings refuse | `cargo run -p mathematics_examples --example precision_boundary_examples` |
| [binary_field_gf2.rs](binary_field_gf2.rs) | `Gf2`, the field with two elements: characteristic 2 and what follows from it, RAID-5 parity and erasure recovery, and Hamming(7,4) single-error correction | `cargo run -p mathematics_examples --example binary_field_gf2_examples` |
| [cayley_dickson_ladder.rs](cayley_dickson_ladder.rs) | `ComplexWitness`, `QuaternionWitness` and `OctonionWitness` carrying the same six traits, so one norm and one zip serve 2, 4 and 8 slots; then what each rung keeps — the multiplicative norm on all three, commutativity through ℂ, associativity through ℍ | `cargo run -p mathematics_examples --example cayley_dickson_ladder_examples` |

## The scalars

Four types implement `Float`, spanning 2.1 to 31.3 decimal digits of resolution:

| Type | Significand bits | Decimal digits |
|---|---|---|
| `BFloat16` | 8 | 2.1 |
| `f32` | 24 | 6.9 |
| `f64` | 53 | 15.7 |
| `Float106` | 106 | 31.3 |

`Float106` holds two `f64` limbs as an unevaluated sum, so it keeps `f64`'s exponent range. It
also serves as the oracle the narrower paths are diffed against.

## The two that are not floats

`Gf2` is a field with two elements, where addition is exclusive-or and every element is its own
additive inverse. It is exact, so no `FloatType` alias appears in its example: precision is a
parameter for scalars that approximate the reals, and there is nothing here to round.

`Quaternion` and `Octonion` extend the reals rather than approximating them differently. Octonion
multiplication derives from the Cayley–Dickson doubling of the quaternions, which carries the
composition law `|xy| = |x||y|` up from the rung below.
