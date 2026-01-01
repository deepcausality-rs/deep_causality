# DFRAFT: DeepCausality GAT-Bounded HKT: The Next Generation

`deep_causality_haft` now supports **GAT-Bounded Higher-Kinded Types** — a principled system for constrained type
constructors that unlocks monadic composition for physics-oriented types like `CausalMultiField`, `SpinorManifold`, and
`GaugeManifold`.

---

## 🏗️ The Problem: Constrained Types Meet HKTs

The original HKT system (see [HAFT.md](HAFT.md)) works beautifully for unconstrained types like `Vec<T>` or `Option<T>`.
But physics types need **bounds**:

```rust
// TensorData requires: Field + Copy + Default + Send + Sync + ...
pub struct CausalMultiField<B: LinearAlgebraBackend, T: TensorData> {
    ...
}

// The HKT trait has NO bounds on T — incompatible!
pub trait HKT {
    type Type<T>;  // T is completely unconstrained
}
```

**The Problem**: We can't implement `HKT` for `CausalMultiFieldWitness` because the GAT `Type<T>` must accept *any* `T`,
but `CausalMultiField` requires `T: TensorData`.

This blocked HKT-based functional programming for:

- Spinor fields (Dirac equation, quantum spin)
- Gauge fields (Yang-Mills, QCD)
- Constrained tensor operations

---

## 🧩 The Solution: `Satisfies<Constraint>`

We introduce a **constraint marker pattern** that lets the *implementor* declare what bounds their type needs:

```rust
// A marker trait: "T satisfies constraint C"
pub trait Satisfies<C: ?Sized> {}

// A marker type representing TensorData bounds
pub struct TensorDataConstraint;

// Blanket impl: any T: TensorData satisfies this constraint
impl<T: TensorData> Satisfies<TensorDataConstraint> for T {}
```

Now we define a new trait that makes the constraint **explicit**:

```rust
pub trait BoundedHKT {
    type Constraint: ?Sized;  // The implementor declares this
    type Type<T>
    where
        T: Satisfies<Self::Constraint>;  // T must satisfy it
}
```

**Now we can implement it:**

```rust
impl<B: LinearAlgebraBackend> BoundedHKT for CausalMultiFieldWitness<B> {
    type Constraint = TensorDataConstraint;
    type Type<T> = CausalMultiField<B, T>
    where
        T: Satisfies<TensorDataConstraint>;  // ✅ Works!
}
```

---

## 📚 The Bounded Trait Hierarchy

The full functional programming stack is now available for constrained types:

| Trait                | Purpose                               | Key Method                                        |
|----------------------|---------------------------------------|---------------------------------------------------|
| `BoundedHKT`         | Declare constraint + type constructor | `type Type<T>`                                    |
| `BoundedFunctor`     | Transform inner values                | `fmap(fa, f) -> fb`                               |
| `BoundedApplicative` | Lift values, apply wrapped functions  | `pure(a)`, `apply(f_ab, f_a)`                     |
| `BoundedMonad`       | Chain computations                    | `bind(m_a, f) -> m_b`                             |
| `BoundedComonad`     | Extract focus, extend to neighbors    | `extract(fa)`, `extend(fa, f)`                    |
| `BoundedAdjunction`  | Translate between domains             | `unit`, `counit`, `left_adjunct`, `right_adjunct` |

### The Functor Laws Still Hold

```rust
// Identity: fmap(fa, id) == fa
let identity = BoundedFunctor::fmap(tensor.clone(), | x| x);
assert_eq!(identity, tensor);

// Composition: fmap(fa, f.compose(g)) == fmap(fmap(fa, g), f)
let direct = BoundedFunctor::fmap(tensor.clone(), | x| (x * 2.0).sqrt());
let composed = BoundedFunctor::fmap(
BoundedFunctor::fmap(tensor, | x| x * 2.0),
| x| x.sqrt()
);
assert_eq!(direct, composed);
```

---

## 🔢 Standard Constraints

The crate provides ready-to-use constraint markers:

| Constraint             | Required Traits          | Use Case                              |
|------------------------|--------------------------|---------------------------------------|
| `NoConstraint`         | None (blanket `impl<T>`) | Fully polymorphic types like `Vec<T>` |
| `NumericConstraint`    | `Zero + Copy + Clone`    | Numeric containers                    |
| `ThreadSafeConstraint` | `Send + Sync`            | Concurrent physics                    |
| `CloneConstraint`      | `Clone`                  | Types requiring cloning               |
| `TensorDataConstraint` | `TensorData`             | Physics field types                   |

### Creating Composite Constraints

Need multiple constraints? Define your own marker:

```rust
// A constraint requiring BOTH TensorData AND thread safety
pub struct TensorDataThreadSafe;

impl<T> Satisfies<TensorDataThreadSafe> for T
where
    T: TensorData + Send + Sync
{}
```

---

## 🌐 Integration with Physics

### The Comonad for Field Operations

`BoundedComonad` is the key abstraction for **local-to-global** operations on physics fields:

```rust
// Heat equation: local Laplacian determines temperature evolution
let evolved_field = BoundedComonad::extend( & temperature_field, | local_view| {
// Extract the local temperature
let center = BoundedComonad::extract(local_view);

// Compute Laplacian from neighbors (uses coboundary operator)
let laplacian = compute_stencil(local_view);

// Return new temperature: T_new = T + dt * κ * ∇²T
center + dt * kappa * laplacian
});
```

### The Adjunction for Integration/Differentiation

`BoundedAdjunction` encodes the duality between chains and cochains:

```rust
// Integration: Chain (path) → Cochain (1-form) → Scalar
let work = BoundedAdjunction::right_adjunct( & context, path, | point| {
force_field.evaluate_at(point)
});

// Differentiation: Scalar field → Vector field (gradient)
let gradient = BoundedAdjunction::left_adjunct( & context, scalar_field, | form| {
form.boundary_operator()
});
```

---

## Constraint Placement: The Orphan Rules

Rust's orphan rules determine **where** you can define constraint markers.

**Rule**: The crate defining the marker struct must also define the blanket `impl`.

| Constraint             | Must Be Defined In                           |
|------------------------|----------------------------------------------|
| `NumericConstraint`    | `deep_causality_haft` (owns `Satisfies`)     |
| `TensorDataConstraint` | `deep_causality_tensor` (owns `TensorData`)  |
| `NumericThreadSafe`    | `deep_causality_haft` (uses only std traits) |
| `TensorDataThreadSafe` | `deep_causality_tensor` (uses `TensorData`)  |

**Decision Tree:**

```
Does your constraint use only std traits (Copy, Clone, Send, Sync)?
├── YES → Define in deep_causality_haft
└── NO → Define in the crate that owns the required trait
```

---

## 🚀 Migration from Legacy Bounded Traits

The old `BoundedComonad` and `BoundedAdjunction` had **hardcoded bounds**:

```rust
// OLD: Bounds embedded in the trait (not universal)
fn extend<A, B, Func>(...) ->...
where
A: Zero + Copy + Clone,  // Hardcoded!
B: Zero + Copy + Clone,  // Hardcoded!
```

The new system replaces hardcoded bounds with the `Satisfies<Constraint>` pattern:

```rust
// NEW: Bounds declared by implementor via BoundedHKT
fn extend<A, B, Func>(...) ->...
where
A: Satisfies<F::Constraint> + Clone,  // From BoundedHKT
B: Satisfies<F::Constraint>,          // From BoundedHKT
```

**Migration is complete replacement** — no backward compatibility, no feature flags.

---

## 🎯 Summary

| Before                                     | After                                             |
|--------------------------------------------|---------------------------------------------------|
| `HKT` — unbounded GAT                      | `BoundedHKT` — constraint declared by implementor |
| `BoundedComonad` — hardcoded `Zero + Copy` | `BoundedComonad` — uses `Satisfies<Constraint>`   |
| `CausalMultiField` — cannot use HKT        | `CausalMultiField` — full HKT support             |

**The Result**: Physics types (`SpinorManifold`, `GaugeManifold`, `CausalMultiField`) now participate in the same
monadic composition framework as simple containers like `Vec` and `Option`.

---

## 📖 Further Reading

- **[HAFT.md](HAFT.md)** — The foundational HKT witness pattern
- **[TOPOLOGY.md](TOPOLOGY.md)** — Comonad operations on manifolds
- **[PHYSICS.md](PHYSICS.md)** — Causal effect propagation
- **[UNIFORM_MATH.md](UNIFORM_MATH.md)** — The unified mathematical framework
- **[specs/current/hkt_gat.md](../specs/current/hkt_gat.md)** — Full technical specification
