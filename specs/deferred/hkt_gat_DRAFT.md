# DeepCausality Unified HKT: One Trait Hierarchy for All Types

`deep_causality_haft` provides a **Unified Higher-Kinded Type (HKT)** system that works for *all* types — from simple `Vec<T>` to physics types requiring `TensorData` bounds — using a single trait hierarchy.

---

## 🏗️ The Problem: Two Worlds

Previously, HKT-based functional programming had a fundamental split:

| World | Types | Limitation |
|-------|-------|------------|
| **Unbounded** | `Vec<T>`, `Option<T>` | Works for any `T` |
| **Bounded** | `CausalTensor<T: TensorData>` | Requires specific bounds on `T` |

These couldn't share traits. You had `Functor` for unbounded types and `BoundedFunctor` for bounded types. **Code duplication. API fragmentation. Confusion.**

---

## 🧩 The Solution: Unified Constraints

**Key insight**: An "unbounded" type is just a "bounded" type with `Constraint = NoConstraint`.

```rust
// The constraint marker system
pub trait Satisfies<C: ?Sized> {}

// NoConstraint: everything satisfies it
pub struct NoConstraint;
impl<T> Satisfies<NoConstraint> for T {}

// TensorDataConstraint: only TensorData types satisfy it
pub struct TensorDataConstraint;
impl<T: TensorData> Satisfies<TensorDataConstraint> for T {}
```

Now the HKT trait is **unified**:

```rust
pub trait HKT {
    type Constraint: ?Sized;  // Implementor declares the constraint
    type Type<T>
    where
        T: Satisfies<Self::Constraint>;
}
```

---

## 📚 The Unified Trait Hierarchy

One set of traits for all types:

```
HKT
 └── Functor
       └── Applicative
             └── Monad

 └── CoMonad (parallel to Monad)

Adjunction (between two HKT functors)
```

| Trait | Key Methods |
|-------|-------------|
| `HKT` | `type Constraint`, `type Type<T>` |
| `Functor<F: HKT>` | `fmap(fa, f) -> fb` |
| `Applicative<F: HKT>` | `pure(a)`, `apply(f_ab, f_a)` |
| `Monad<F: HKT>` | `bind(m_a, f)`, `join(m_m_a)` † |
| `CoMonad<F: HKT>` | `extract(fa)`, `extend(fa, f)`, `duplicate(fa)` † |
| `Adjunction<L, R: HKT>` | `unit`, `counit`, `left_adjunct`, `right_adjunct` |

† Default implementation provided

---

## 🔧 Usage Examples

### Unconstrained Type: Vec

```rust
impl HKT for VecWitness {
    type Constraint = NoConstraint;  // Everything works!
    type Type<T> = Vec<T>
    where
        T: Satisfies<NoConstraint>;
}

impl Functor<VecWitness> for VecWitness {
    fn fmap<A, B, F>(fa: Vec<A>, mut f: F) -> Vec<B>
    where
        A: Satisfies<NoConstraint>,  // Always true
        B: Satisfies<NoConstraint>,  // Always true
        F: FnMut(A) -> B,
    {
        fa.into_iter().map(f).collect()
    }
}
```

### Constrained Type: CausalTensor

```rust
impl HKT for CausalTensorWitness {
    type Constraint = TensorDataConstraint;  // Only TensorData types
    type Type<T> = CausalTensor<T>
    where
        T: Satisfies<TensorDataConstraint>;
}

impl Functor<CausalTensorWitness> for CausalTensorWitness {
    fn fmap<A, B, F>(fa: CausalTensor<A>, mut f: F) -> CausalTensor<B>
    where
        A: Satisfies<TensorDataConstraint>,  // A: TensorData
        B: Satisfies<TensorDataConstraint>,  // B: TensorData
        F: FnMut(A) -> B,
    {
        let shape = fa.shape().to_vec();
        CausalTensor::new(fa.into_vec().into_iter().map(f).collect(), shape).unwrap()
    }
}
```

### Generic Over Any HKT

```rust
fn double_all<F: HKT>(container: F::Type<f64>) -> F::Type<f64>
where
    F::Type<f64>: Clone,
    f64: Satisfies<F::Constraint>,
    VecWitness: Functor<F>,
{
    Functor::<F>::fmap(container, |x| x * 2.0)
}

// Works for Vec<f64>, CausalTensor<f64>, or any other HKT with f64-compatible constraint
```

---

## 🔢 Standard Constraints

| Constraint | Required Traits | Defined In |
|------------|-----------------|------------|
| `NoConstraint` | None | `haft` |
| `NumericConstraint` | `Zero + Copy + Clone` | `haft` |
| `ThreadSafeConstraint` | `Send + Sync` | `haft` |
| `CloneConstraint` | `Clone` | `haft` |
| `TensorDataConstraint` | `TensorData` | `tensor` |

### Creating Composite Constraints

```rust
pub struct TensorDataThreadSafe;
impl<T: TensorData + Send + Sync> Satisfies<TensorDataThreadSafe> for T {}
```

---

## 🌐 Physics Integration

### CoMonad for Field Operations

The unified `CoMonad` works for both simple containers and physics fields:

```rust
// Heat diffusion on a manifold
let evolved = CoMonad::<ManifoldWitness>::extend(&temperature_field, |local| {
    let center = CoMonad::<ManifoldWitness>::extract(local);
    let laplacian = compute_laplacian(local);
    center + dt * kappa * laplacian
});
```

### Adjunction for Calculus

```rust
// Integration (right adjunct) and differentiation (left adjunct)
let work = Adjunction::<ChainWitness, CochainWitness, _>::right_adjunct(
    &ctx, path, |point| force_field.at(point)
);
```

---

## ⚠️ Orphan Rules

The crate defining the marker must define the blanket impl:

| Constraint | Must Be In |
|------------|------------|
| `NumericConstraint` | `deep_causality_haft` |
| `TensorDataConstraint` | `deep_causality_tensor` |

---

## 🎯 What Changed

| Before | After |
|--------|-------|
| `HKT` (unbounded) + `BoundedHKT` (proposed) | Single `HKT` with `type Constraint` |
| `Functor` + `BoundedFunctor` | Single `Functor` |
| `CoMonad` + `BoundedComonad` | Single `CoMonad` |
| `Adjunction` + `BoundedAdjunction` | Single `Adjunction` |
| Hardcoded bounds (`Zero + Copy`) | Declarative constraints |
| `CausalMultiField` couldn't use HKT | Full HKT support ✅ |

---

## 📖 Further Reading

- **[HAFT.md](../../docs/HAFT.md)** — Original HKT concepts (witness pattern)
- **[TOPOLOGY.md](../../docs/TOPOLOGY.md)** — CoMonad on manifolds
- **[PHYSICS.md](../../docs/PHYSICS.md)** — Causal effect propagation
- **[specs/current/hkt_gat.md](hkt_gat.md)** — Full technical specification
