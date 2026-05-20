# Iso Impact on Examples

Where each of the six concrete isos in `implement-isomorphism` would (or would not) simplify code in [`examples/`](../../examples/). Scanned via `grep` for the relevant carrier types across all example crates.

## Summary table

| Iso | Examples helped | Magnitude | Notes |
|---|---|---|---|
| Quaternion <-> Cl(3,0)-even rotor | `triple_hkt_stress_field`, `effect_tensor_algebra_roundtrip` | Real (5-8 LoC saved per rotor site) | Both examples build rotors by hand-packing 8-element coefficient vectors |
| CausalTensor (rank-2) <-> CsrMatrix | None today | Future-facing only | No example currently materialises a sparse output to dense |
| Complex <-> Cl(0,1) multivector | None | None | All multivector code in examples uses Cl(2+); no pure Cl(0,1) carrier exists in the example corpus |
| CausalMultiField <-> tuple | None | None | `CausalMultiField` is not used anywhere under `examples/` |
| SimplicialComplex <-> CellComplex<Simplex> | None | None | Examples build `SimplicialComplex` directly; no cell-complex consumer code |
| LatticeComplex<D> <-> DualLatticeComplex<D> (Poincaré) | None | None | No example constructs a `DualLatticeComplex` |

The propagating-effect consistency test (no iso wrapper) doesn't apply to examples — it pins library-internal consistency.

---

## Concrete simplifications

### Quaternion <-> Cl(3,0)-even rotor

**Two sites in two examples.** Both construct a 10-degree (or 90-degree) Clifford rotor by manually packing the scalar and `e1^e2` bivector coefficients into the right slots of an 8-element vector. The iso replaces this with a quaternion constructor.

#### Site A: [`mathematics_examples/triple_hkt_stress_field/main.rs:364-381`](../../examples/mathematics_examples/triple_hkt_stress_field/main.rs#L364-L381)

```rust,ignore
// BEFORE — material_rotor()
let metric = Metric::Euclidean(3);
let theta: FloatType = 10.0_f64.to_radians();
let c = (theta / 2.0).cos();
let s = (theta / 2.0).sin();

let mut r = vec![0.0; 8];
r[0] = c;
r[3] = -s; // -sin(theta/2) * e1^e2
let rotor = CausalMultiVector::new(r, metric).unwrap();

let mut r_rev = vec![0.0; 8];
r_rev[0] = c;
r_rev[3] = s;
let rotor_rev = CausalMultiVector::new(r_rev, metric).unwrap();
```

```rust,ignore
// AFTER — material_rotor()
let theta: FloatType = 10.0_f64.to_radians();
let c = (theta / 2.0).cos();
let s = (theta / 2.0).sin();

// East-coast convention: k corresponds to e1^e2
let rotor: CausalMultiVector<f64> = Quaternion::new(c, 0.0, 0.0, -s).into();
let rotor_rev: CausalMultiVector<f64> = Quaternion::new(c, 0.0, 0.0, s).into();
```

Savings: ~8 LoC of manual basis-index bookkeeping, no `.unwrap()`, no chance of putting a sign on the wrong bivector slot. The "east-coast convention" comment is the only domain-knowledge requirement; the rest is mechanical.

#### Site B: [`mathematics_examples/effect_tensor_algebra_roundtrip/main.rs:90-101`](../../examples/mathematics_examples/effect_tensor_algebra_roundtrip/main.rs#L90-L101)

Same pattern: 90-degree rotation in the `e1^e2` plane built by hand-packing a length-8 vector. Same replacement, same ~8 LoC saved.

```rust,ignore
// AFTER — rotate_in_xy()
let theta = FloatType::pi() / FloatType::from(2.0);
let half = theta / FloatType::from(2.0);
let c = half.cos();
let sn = half.sin();
let rotor: CausalMultiVector<f64> = Quaternion::new(c, 0.0, 0.0, -sn).into();
let rotor_rev: CausalMultiVector<f64> = Quaternion::new(c, 0.0, 0.0, sn).into();
```

#### What this iso does *not* simplify

- The grade-1 lift in `effect_tensor_algebra_roundtrip::lift_to_algebra` (a 3-vector padded into an 8-element multivector). That's a vector embedding, not a rotor; the Quaternion iso doesn't apply.
- The rotor in `tensor_x_algebra_rotation_field` uses **Cl(2,0)** (4-element basis), not Cl(3,0). The Cl(2,0)-even subalgebra is also isomorphic to ℂ, but the iso we're shipping targets Cl(3,0)-even specifically. A separate `Complex <-> Cl(2,0)-even` iso would simplify this site, but it isn't in the current change.

### CausalTensor <-> CsrMatrix (future-facing)

No example today materialises a sparse output to dense or vice versa. The closest patterns are:

- `capstone_spinor_minkowski`, `tensor_x_topology_laplacian`, `effect_diffusion_on_manifold`: each builds a `CsrMatrix` via `from_triplets` AND constructs a parallel `CausalTensor::new(data, ...)`. The two carriers run in parallel; no conversion happens between them.

So while the iso adds no LoC savings to current code, the `to_dense()` ergonomic alias would be available for any future example that operates on sparse outputs (Laplacian eigensolves, graph spectral decompositions, etc.) and wants to feed them into dense pipelines.

---

## What's missing from the example corpus

The audit revealed two iso pairs in the change with effectively zero example coverage:

- **CausalMultiField**: zero uses across all examples. The multifield carrier is library-internal infrastructure with no public example yet. Landing the iso is fine, but its value lives in future code.
- **DualLatticeComplex**: zero uses. The Poincaré iso is mathematically the most interesting addition; it has no example consumer today.

This is not an argument against shipping the isos; it's a flag for the follow-up work that *should* exist. Two candidate example additions worth considering after `implement-isomorphism` lands:

1. **A Poincaré-dual demo in `physics_examples/`**: take a 3D lattice with a scalar field, dualise via `PoincareIso<3>`, run a Hodge-star operation, dualise back. This is the canonical worked example for cubical-Regge / lattice-gauge code.
2. **A multifield/tensor demo in `mathematics_examples/`**: pack a `CausalMultiField` from a tensor, run a few differential ops, unpack back. Currently the multifield API is exercised only in unit tests; a public example would close that gap.

Neither is in scope for `implement-isomorphism`; both are reasonable follow-ups once the surface exists.

---

## What this audit does *not* claim

- That every iso in the change "pays its way" in current examples. Four of six don't, and two of those (multifield/tensor, Poincaré) wouldn't even have a downstream consumer in any example today. They're shipped because they're mathematically clean and have real library-internal value, not because they shorten example code.
- That the Quaternion iso's two sites are a sufficient reason to ship the iso alone. The iso would land for the marker-subtrait coverage value (full `DivisionAlgebraIso` exercise) regardless of example simplification.

The honest framing: **the Quaternion iso has measurable example-code value; the other five are foundation work whose value will show up in subsequent example or pipeline additions**.
