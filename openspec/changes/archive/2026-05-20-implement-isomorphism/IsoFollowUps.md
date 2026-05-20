# Iso Follow-Up Designs

Follow-up notes for the three-tier iso change archived at
`openspec/changes/archive/2026-05-20-2026-05-20-add-iso-traits/`. Each
section sketches one concrete iso instance that can land as its own
change. Both are intentionally scoped per-crate so the diffs stay local.

---

## 1. `PropagatingEffect<T>` <-> `PropagatingProcess<T, (), ()>` (Tier 3, `NaturalIso`)

### What's actually being related

Both surfaces are type aliases over the same 5-arity carrier:

```rust,ignore
// deep_causality_core/src/types/propagating_effect/mod.rs
pub type PropagatingEffect<T> =
    CausalEffectPropagationProcess<T, (), (), CausalityError, EffectLog>;

// deep_causality_core/src/types/propagating_process/mod.rs
pub type PropagatingProcess<T, S, C> =
    CausalEffectPropagationProcess<T, S, C, CausalityError, EffectLog>;
```

Set `S = ()`, `C = ()` and `PropagatingProcess<T, (), ()>` and
`PropagatingEffect<T>` are the literally same concrete type. The
relationship is structural type equality at the value level.

The reason we still need an explicit iso: each surface ships its own
HKT witness with its own `Functor` / `Applicative` / `Monad` impls.

```rust,ignore
// hkt.rs in propagating_effect:
pub struct PropagatingEffectWitness<E, L>(...);
impl<E, L> HKT for PropagatingEffectWitness<E, L> {
    type Type<T> = CausalEffectPropagationProcess<T, (), (), E, L>;
}

// hkt.rs in propagating_process:
pub struct PropagatingProcessWitness<S, C>(...);
impl<S, C> HKT for PropagatingProcessWitness<S, C> {
    type Type<T> = PropagatingProcess<T, S, C>;
}
```

Pick `E = CausalityError, L = EffectLog` and `S = (), C = ()` and the
two `Type<T>` projections produce the same carrier. Generic code
parameterised over one witness cannot today be reused against the
other; we want a single iso declaration to fix that.

### The iso

Tier 3 fits exactly because we are relating two HKT witnesses, not two
concrete types. Arity 1 is enough; the higher arities aren't needed
because we only vary `T`.

```rust,ignore
// proposed: deep_causality_core/src/types/iso/effect_process_iso.rs

use deep_causality_haft::{NaturalIso, NoConstraint, Satisfies};

use crate::{
    CausalityError, EffectLog, PropagatingEffectWitness, PropagatingProcessWitness,
};

/// Natural iso between the propagating-effect carrier and the trivial
/// (unit-state, unit-context) propagating-process carrier.
///
/// Both witnesses project to the same `CausalEffectPropagationProcess
/// <T, (), (), CausalityError, EffectLog>` carrier, so the iso is the
/// identity at the value level. Declaring it lifts the equality to
/// the witness level so generic functor / monad code can move between
/// the two witnesses without bespoke wrappers.
pub struct EffectProcessIso;

impl
    NaturalIso<
        PropagatingEffectWitness<CausalityError, EffectLog>,
        PropagatingProcessWitness<(), ()>,
    > for EffectProcessIso
{
    fn to_target<T>(
        fa: <PropagatingEffectWitness<CausalityError, EffectLog> as HKT>::Type<T>,
    ) -> <PropagatingProcessWitness<(), ()> as HKT>::Type<T>
    where
        T: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
    {
        // Same concrete type on both sides; identity.
        fa
    }

    fn to_source<T>(
        ga: <PropagatingProcessWitness<(), ()> as HKT>::Type<T>,
    ) -> <PropagatingEffectWitness<CausalityError, EffectLog> as HKT>::Type<T>
    where
        T: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
    {
        ga
    }
}
```

The body of both methods is identity. The trait impl is paperwork; its
value is the type-level promise.

### Laws to pin

Both laws hold trivially because every conversion is identity. The
test still goes in because a future refactor that puts a real
transformation behind one of the witnesses must not silently break
the iso.

```rust,ignore
#[test]
fn effect_process_iso_round_trip() {
    use deep_causality_haft::iso::test_support::assert_natural_iso_round_trip;
    let eff = PropagatingEffectWitness::<CausalityError, EffectLog>::pure(42);
    let proc = PropagatingProcessWitness::<(), ()>::pure(42);
    assert_natural_iso_round_trip::<
        EffectProcessIso,
        PropagatingEffectWitness<CausalityError, EffectLog>,
        PropagatingProcessWitness<(), ()>,
        i32,
    >(eff, proc);
}

#[test]
fn effect_process_iso_naturality_doubling() {
    use deep_causality_haft::iso::test_support::assert_natural_iso_naturality;
    let eff = PropagatingEffectWitness::<CausalityError, EffectLog>::pure(3);
    assert_natural_iso_naturality::<
        EffectProcessIso,
        PropagatingEffectWitness<CausalityError, EffectLog>,
        PropagatingProcessWitness<(), ()>,
        i32,
        i32,
        _,
    >(eff, |x| x * 2);
}
```

The naturality test does work: it exercises both `fmap` impls (the
ones on `PropagatingEffectWitness` and on `PropagatingProcessWitness`)
and confirms they agree on the shared carrier. Today these impls live
in two separate files and were written independently; the test pins
that they stay consistent.

### Scope of the follow-up change

* New module `deep_causality_core/src/types/iso/` with `effect_process_iso.rs`.
* Re-export `EffectProcessIso` from `lib.rs`.
* Tests in `deep_causality_core/tests/iso/` (round-trip + naturality + a
  couple of representative `fmap` / `bind` cases proving generic
  pipelines work against either witness).
* Bazel test-suite entry under `deep_causality_core/tests/BUILD.bazel`.
* No changes to the surfaces of `PropagatingEffect` or `PropagatingProcess`.

### Why bother

Two real consumers benefit:

1. **CDL pipeline**. The discovery DSL sits on top of `PropagatingEffect`
   while the simulation / replay machinery uses `PropagatingProcess<T, S, C>`
   with non-trivial `S`. With the iso, helper code written for one path can
   be lifted via `to_target` to the other without rewriting; the type
   system enforces that nothing was dropped.
2. **Effect-system migrations**. When we add a new variant of the carrier
   (e.g. a logging-only specialisation) the iso vocabulary lets us
   declare equivalences and reuse pipelines without retesting them
   end-to-end.

Open question (not blocking): whether to ship a higher-arity
`EffectProcessIso5` against `NaturalIso5` that maps the full 5-arity
witness shape so `S` and `C` aren't pinned to `()`. The arity-5
machinery exists; the worked example just hasn't shown up yet.

---

## 2. `CausalTensor<F>` <-> `CsrMatrix<F>` (mixed-tier: Tier 1 forward, Tier 2 reverse)

### The orphan-rule problem

Dependency direction: `deep_causality_sparse` depends on
`deep_causality_tensor` (sparse reads dense), not the other way. So:

| Direction | Possible? | Where? |
|---|---|---|
| `impl From<CausalTensor<F>> for CsrMatrix<F>` | yes | in `deep_causality_sparse` (Self is local) |
| `impl From<CsrMatrix<F>> for CausalTensor<F>` | NO | orphan rule: Self foreign, `F` uncovered |

Tier 1 needs both `From` directions to wire `GroupIso<T>` etc., so a
pure-Tier-1 iso is impossible here. The mixed-tier pattern from
[NumIso.md §6] is the fit: ship the forward direction as a Tier 1
`From`; ship the reverse direction as a Tier 2 `Iso<CsrMatrix<F>,
CausalTensor<F>>` impl on `CsrMatrix<F>` as `Self`.

### Forward: `From<CausalTensor<F>> for CsrMatrix<F>`

```rust,ignore
// proposed: deep_causality_sparse/src/types/sparse_matrix/from_tensor.rs

use deep_causality_num::Zero;
use deep_causality_tensor::CausalTensor;

use crate::CsrMatrix;

impl<F> From<CausalTensor<F>> for CsrMatrix<F>
where
    F: Zero + PartialEq + Clone,
{
    fn from(tensor: CausalTensor<F>) -> Self {
        // Only sensible for rank-2 tensors. Rank mismatch is a runtime
        // precondition (panic) because `From` cannot express it in the
        // type system; callers reach for `TryFrom` if they need
        // graceful failure.
        let shape = tensor.shape();
        assert_eq!(
            shape.len(),
            2,
            "CausalTensor -> CsrMatrix requires rank 2, got rank {}",
            shape.len()
        );
        let rows = shape[0];
        let cols = shape[1];

        let mut row_indices = Vec::new();
        let mut col_indices = Vec::new();
        let mut values = Vec::new();
        let data = tensor.into_data();

        for r in 0..rows {
            for c in 0..cols {
                let v = data[r * cols + c].clone();
                if !v.is_zero() {
                    row_indices.push(r);
                    col_indices.push(c);
                    values.push(v);
                }
            }
        }

        CsrMatrix::from_triplets(row_indices, col_indices, values, (rows, cols))
    }
}
```

(Names like `into_data` and `from_triplets` are placeholders; real
constructors come from the existing `CausalTensor` / `CsrMatrix` APIs
or are added alongside the iso.)

### Reverse: Tier 2 `Iso<CsrMatrix<F>, CausalTensor<F>>` on `CsrMatrix<F>`

`CsrMatrix<F>` is local to `deep_causality_sparse`, so a Tier 2 impl
with `Self = CsrMatrix<F>` is orphan-rule-safe.

```rust,ignore
// proposed: deep_causality_sparse/src/types/sparse_matrix/iso.rs

use deep_causality_num::Zero;
use deep_causality_num::iso::witness::Iso;
use deep_causality_tensor::CausalTensor;

use crate::CsrMatrix;

impl<F> Iso<CsrMatrix<F>, CausalTensor<F>> for CsrMatrix<F>
where
    F: Zero + Clone,
{
    fn to_target(s: CsrMatrix<F>) -> CausalTensor<F> {
        let (rows, cols) = s.shape();
        let mut data = vec![F::zero(); rows * cols];
        for (r, c, v) in s.into_triplets() {
            data[r * cols + c] = v;
        }
        CausalTensor::from_vec(data, vec![rows, cols])
            .expect("rows * cols matches data length by construction")
    }

    fn to_source(t: CausalTensor<F>) -> CsrMatrix<F> {
        // Reuse the forward `From` impl on the same edge.
        CsrMatrix::from(t)
    }
}
```

### Convenience: inherent `to_dense()` alias

Because Tier 2 lives on the trait, ergonomic call sites still want a
method form. Add an inherent alias on `CsrMatrix<F>` that delegates to
the iso:

```rust,ignore
impl<F> CsrMatrix<F>
where
    F: Zero + Clone,
{
    /// Materialise this sparse matrix as a dense rank-2
    /// [`CausalTensor`]. Equivalent to
    /// `<Self as Iso<CsrMatrix<F>, CausalTensor<F>>>::to_target(self)`.
    pub fn to_dense(self) -> CausalTensor<F> {
        <Self as Iso<CsrMatrix<F>, CausalTensor<F>>>::to_target(self)
    }
}
```

Idiomatic call sites become:

```rust,ignore
let sparse: CsrMatrix<f64> = tensor.into();   // forward (From)
let dense:  CausalTensor<f64> = sparse.to_dense();  // reverse (Iso + alias)
```

### Why no Tier 2 marker (GroupIso, RingIso, ...)

`CsrMatrix<F>` and `CausalTensor<F>` are not algebraic structures in
the `deep_causality_num` sense. Neither implements `Group`, `Ring`, or
`Field` (matrix algebra is a multiplicative structure but the way our
tensor / sparse APIs are shaped today, the algebraic-trait impls are
not present). So the base `Iso<S, T>` is the right surface; the marker
subtraits would never apply. `StandardIso<S, T>` is also not usable
here because bidirectional `From` does not exist (the whole point of
this section).

### Laws to pin

Round-trip in both directions, with the independent-input discipline
the Tier 2 helper enforces:

```rust,ignore
#[test]
fn tensor_sparse_iso_round_trip() {
    use deep_causality_num::iso::witness::test_support::assert_witness_iso_round_trip;

    let dense = CausalTensor::from_vec(
        vec![1.0, 0.0, 0.0, 4.0, 0.0, 6.0],
        vec![2, 3],
    )
    .unwrap();
    let sparse = CsrMatrix::from_triplets(
        vec![0, 1, 1],
        vec![0, 0, 2],
        vec![1.0, 4.0, 6.0],
        (2, 3),
    );

    assert_witness_iso_round_trip::<CsrMatrix<f64>, CsrMatrix<f64>, CausalTensor<f64>>(
        sparse, dense,
    );
}
```

Plus a `#[should_panic]` test for rank-mismatch on the forward
direction (`From` of a rank-3 tensor) so the assertion is locked in.

### Scope of the follow-up change

* New module `deep_causality_sparse/src/types/sparse_matrix/iso.rs`
  with the Tier 2 impl plus the inherent `to_dense()` alias.
* New module `deep_causality_sparse/src/types/sparse_matrix/from_tensor.rs`
  with the Tier 1 forward `From` impl.
* Re-exports from `deep_causality_sparse/src/lib.rs`.
* Tests under `deep_causality_sparse/tests/iso/` covering forward,
  reverse, round-trip, and rank-mismatch panic.
* Bazel test-suite entry.
* Zero changes in `deep_causality_tensor`.

### Why bother

Two concrete consumers downstream:

1. **CDL pipeline**. Discovery operates on dense tensors;
   intermediate results often arrive sparse (correlation thresholding,
   conditional independence tests). Today the conversion is hand-rolled
   in every pipeline stage. The iso centralises it.
2. **Memory budget**. Several physics-domain analyses (e.g. the
   propagating-effect work in `3DCausalFluidDynamics.md`) want to
   move large coefficient matrices through dense ops then store them
   sparse. `tensor.into()` followed later by `sparse.to_dense()` is
   the natural form.

### Open questions

* **`TryFrom` vs panicking `From`.** The forward direction is partial
  (rank-2-only). Panic on rank mismatch matches the rest of the tensor
  API but a `TryFrom` form would compose better with the result-typed
  pipeline. Decision can be deferred; both can coexist.
* **Symmetric markers.** If `CausalTensor` and `CsrMatrix` ever grow
  `Group<+>` or `Module<F>` impls (matrix addition, scalar scaling),
  the corresponding Tier 2 marker subtraits should be added alongside
  to lock the homomorphism property.

---

## Sequencing

Either follow-up can land independently of the other. The
`EffectProcessIso` is smaller (identity bodies) and a good first
exercise of Tier 3 against real consumer types. The tensor / sparse
iso is the canonical mixed-tier worked example and exercises the
`to_dense()` ergonomics pattern that other cross-crate isos will
copy.

If both land, the mixed-tier pattern in tensor/sparse establishes
the template; subsequent isos (e.g. `Quaternion<F>` <->
`CausalMultiVector<F>` in the multivector crate) follow the same
shape.
