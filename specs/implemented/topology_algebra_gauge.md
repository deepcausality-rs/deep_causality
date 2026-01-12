# Algebraic Trait Bounds for Gauge Field Types

**Status:** Approved  
**Author:** DeepCausality Team  
**Created:** 2026-01-12  
**Target Crate:** `deep_causality_topology`

---

## Executive Summary

Migrate the gauge field types (`GaugeField`, `LatticeGaugeField`, `LinkVariable`) from ad-hoc trait bounds (
`TensorData`, `Float`) to algebraic trait bounds from `deep_causality_num`. This enables:

1. **Complex U(1) support** — Proper Monte Carlo sampling with `Complex<f64>`
2. **Type safety** — Algebraic properties encoded at compile time
3. **Unified API** — Same code works for real and complex matrix elements
4. **Mathematical rigor** — Trait bounds reflect actual mathematical requirements

---

## Problem Statement

### Current State

The gauge field implementation uses `Float` as the trait bound for matrix elements:

```rust
// Current: LinkVariable
pub struct LinkVariable<G: GaugeGroup, T> {
    data: CausalTensor<T>,  // T: TensorData (too permissive)
}

impl<G: GaugeGroup, T: TensorData> LinkVariable<G, T> {
    pub fn try_identity() -> Result<Self, Error>
    where
        T: Float,  // Problem: Float not implemented for Complex<f64>
}
```

### The Bug

U(1) gauge theory requires complex phases `e^{iθ}`, but:

| Expected                              | Actual                                         |
|---------------------------------------|------------------------------------------------|
| `LinkVariable<U1, Complex<f64>>`      | Won't compile (`Float` not impl for `Complex`) |
| U(1) elements are `e^{iθ} ∈ U(1) ⊂ ℂ` | Only `±1.0 ∈ ℝ` possible with real `f64`       |
| Metropolis explores phase space       | Stuck at identity (no ergodicity)              |

### Evidence

```
test_thermalization_from_hot_start: FAILED
  Acceptance rate = 100% (all proposals identical)
  Plaquette stays at 0 (no thermalization)
```

---

## Solution Overview

### Two-Parameter Type System

Introduce **matrix element type `M`** and **scalar type `R`** with proper algebraic bounds:

```rust
// New: LinkVariable with algebraic bounds
pub struct LinkVariable<G: GaugeGroup, M, R> {
    data: CausalTensor<M>,  // M: matrix elements (real or complex)
    _scalar: PhantomData<R>, // R: scalar outputs (action, beta)
}

// Bounds
where
M: DivisionAlgebra<R> + Field,  // Matrix elements form a division algebra
R: RealField,                    // Scalars are real (for exp, sin, etc.)
```

### Trait Correspondence

| Operation           | Trait Required                     | Methods Used            |
|---------------------|------------------------------------|-------------------------|
| Matrix multiply     | `Mul<Output=Self>`                 | From `Field` via `Ring` |
| Hermitian conjugate | `DivisionAlgebra::conjugate()`     | `U† = U*`               |
| Trace (real part)   | `ComplexField::real()` or identity | `Re[Tr(U)]`             |
| Action computation  | `RealField::exp()`                 | `e^{-ΔS}` in Metropolis |
| Random phases       | `ComplexField::from_polar()`       | `e^{iθ}` for U(1)       |

---

## Detailed Design

### Design Principles

1. **No Backward Compatibility** — Update downstream type aliases directly; no deprecation wrappers
2. **Method-Level Trait Bounds** — Place bounds on methods, not impl blocks, to allow methods with less strict bounds to remain unaffected
3. **Two-Parameter Pattern** — Established best practice in this codebase for disentangling complex type bounds

### Type Parameter Changes

#### Before

```rust
pub struct LatticeGaugeField<G: GaugeGroup, const D: usize, T> {
    links: HashMap<LatticeCell<D>, LinkVariable<G, T>>,
    beta: T,
}
```

#### After

```rust
// Struct has minimal bounds (or none on M, R)
pub struct LatticeGaugeField<G: GaugeGroup, const D: usize, M, R> {
    links: HashMap<LatticeCell<D>, LinkVariable<G, M, R>>,
    beta: R,  // Coupling is always real
}

// Bounds on METHODS, not impl block
impl<G: GaugeGroup, const D: usize, M, R> LatticeGaugeField<G, D, M, R> {
    // Basic getters - no bounds needed
    pub fn beta(&self) -> &R { &self.beta }
    
    // Methods requiring algebraic ops - bounds on method
    pub fn try_wilson_action(&self) -> Result<R, Error>
    where
        M: DivisionAlgebra<R> + Field,
        R: RealField,
    { ... }
}
```

### Key Type Instances

| Use Case         | M (Matrix)     | R (Scalar)    | Example                    |
|------------------|----------------|---------------|----------------------------|
| Real SU(2)/SU(3) | `f64`          | `f64`         | QCD at high β              |
| Complex U(1)     | `Complex<f64>` | `f64`         | QED, exact Bessel solution |
| High-precision   | `DoubleFloat`  | `DoubleFloat` | Near-continuum physics     |
| Complex SU(N)    | `Complex<f64>` | `f64`         | Full lattice QCD           |

---

## File-by-File Migration

### Phase 1: Core Types

#### 1.1 `link_variable/mod.rs`

**Current:**

```rust
pub struct LinkVariable<G: GaugeGroup, T> {
    data: CausalTensor<T>,
    _gauge: PhantomData<G>,
}
```

**Target:**

```rust
// NO bounds on struct definition
pub struct LinkVariable<G: GaugeGroup, M, R> {
    data: CausalTensor<M>,
    _gauge: PhantomData<G>,
    _scalar: PhantomData<R>,
}

// Bounds on methods that need them
impl<G: GaugeGroup, M, R> LinkVariable<G, M, R> {
    // No bounds for basic getters
    pub fn as_slice(&self) -> &[M] { self.data.as_slice() }
    
    // Bounds only where needed
    pub fn try_identity() -> Result<Self, Error>
    where
        M: Field,
        R: RealField,
    { ... }
    
    pub fn dagger(&self) -> Self
    where
        M: DivisionAlgebra<R>,
    { ... }
}
```

**Changes:**

- Add type parameter `R`
- Replace `T: Float` bounds with `M: DivisionAlgebra<R>, R: RealField`
- Update all method signatures

#### 1.2 `link_variable/ops.rs`

**Key Method Changes:**

| Method              | Before                 | After                                            |
|---------------------|------------------------|--------------------------------------------------|
| `dagger()`          | Transpose only         | Use `DivisionAlgebra::conjugate()` per element   |
| `re_trace()`        | `self.trace()`         | `self.trace().real()` or identity for real types |
| `frobenius_norm_sq` | `val * val` (sum)      | `val.norm_sqr()` (returns R, not M)              |
| `project_sun()`     | Newton-Schulz on reals | Handle complex norm properly using R             |

**Norm Handling Trap:**
- `frobenius_norm_sq` must use `val.norm_sqr()` (from `DivisionAlgebra`) to ensure it returns a real scalar `R` (e.g., $|z|^2$).
- Using `val * val` for complex numbers results in $z^2$ (complex), which is physically incorrect for a norm.
- `project_sun` normalization logic must operate on `R`.

**Dagger Implementation:**

```rust
// Before: Only transpose (wrong for complex)
for i in 0..n {
for j in 0..n {
result[j * n + i] = slice[i * n + j];
}
}

// After: Transpose + conjugate
for i in 0..n {
for j in 0..n {
result[j * n + i] = slice[i * n + j].conjugate();
}
}
```

#### 1.3 `link_variable/random.rs`

**Requirement:**
- Use `RandomField` trait to bridge `deep_causality_rand` and algebraic types.
- `LinkVariable::try_random` requires `M: RandomField`.
- `RandomField` implementation ensures uniform generation in $[-0.5, 0.5]$ for both real and complex components.

#### 1.4 `gauge_field_lattice/mod.rs`

**Current:**

```rust
pub struct LatticeGaugeField<G: GaugeGroup, const D: usize, T> {
    lattice: Arc<Lattice<D>>,
    links: HashMap<LatticeCell<D>, LinkVariable<G, T>>,
    beta: T,
}
```

**Target:**

```rust
pub struct LatticeGaugeField<G: GaugeGroup, const D: usize, M, R>
where
    M: DivisionAlgebra<R> + Field,
    R: RealField,
{
    lattice: Arc<Lattice<D>>,
    links: HashMap<LatticeCell<D>, LinkVariable<G, M, R>>,
    beta: R,  // β is always real
}
```

---

### Phase 2: Operations

#### 2.1 `ops_metropolis.rs`

**Critical Fix for U(1):**

The proposal generation must use complex phases for U(1):

```rust
// Before (broken for U(1))
fn generate_small_su_n_update(&self, epsilon: T, rng: &mut R) -> Result<LinkVariable<G, T>, Error>
where
    T: Float
{
    // Adds real perturbation, projects → only ±1 possible
}

// After (works for U(1))
fn generate_small_su_n_update<R: Rng>(&self, epsilon: R, rng: &mut Rand) -> Result<LinkVariable<G, M, R>, Error>
where
    M: ComplexField<R>,
    R: RealField,
{
    // For U(1): generate e^{iδ} where δ ~ Uniform(-ε, ε)
    if G::matrix_dim() == 1 && G::IS_ABELIAN {
        // U(1) special case: random phase
        let delta: R = R::from(2.0 * rng.random::<f64>() - 1.0).unwrap() * epsilon;
        let phase = M::from_polar(R::one(), delta);  // e^{iδ}
        return Ok(LinkVariable::from_scalar(phase));
    }
    // General SU(N) case...
}
```

#### 2.2 `ops_actions.rs`

**Action Uses Real Scalar:**

```rust
// Action is always real
pub fn try_wilson_action(&self) -> Result<R, TopologyError>
where
    M: DivisionAlgebra<R>,
    R: RealField,
{
    // S = β * Σ_p (1 - Re[Tr(U_p)] / N)
    let mut action = R::zero();
    for plaquette in self.plaquettes() {
        let trace = plaquette.re_trace();  // Returns R
        action += self.beta * (R::one() - trace / R::from(G::matrix_dim()).unwrap());
    }
    Ok(action)
}
```

#### 2.3 `ops_monte_carlo.rs`

**Local Action Change:**

```rust
pub fn try_local_action_change(
    &self,
    edge: &LatticeCell<D>,
    new_link: &LinkVariable<G, M, R>,
) -> Result<R, TopologyError>
where
    M: DivisionAlgebra<R>,
    R: RealField,
{
    // ΔS ∈ R, even if links are complex
    let old_link = self.get_link_or_identity(edge);
    let staple = self.try_staple(edge)?;

    let old_tr: R = old_link.mul(&staple.dagger()).re_trace();
    let new_tr: R = new_link.mul(&staple.dagger()).re_trace();

    Ok(*self.beta() * (old_tr - new_tr) / R::from(G::matrix_dim()).unwrap())
}
```

---

### Phase 3: GaugeField (Continuum)

#### 3.1 `gauge_field/mod.rs`

**Current:**

```rust
pub struct GaugeField<G: GaugeGroup, T, A, F> {
    base: Manifold<T, T>,
    connection: CausalTensor<A>,
    field_strength: CausalTensor<F>,
}
```

**Target:**

```rust
pub struct GaugeField<G: GaugeGroup, M, R>
where
    M: DivisionAlgebra<R>,
    R: RealField,
{
    base: Manifold<R, R>,           // Base manifold uses real coords
    connection: CausalTensor<M>,     // A_μ can be complex
    field_strength: CausalTensor<M>, // F_μν can be complex
}
```

**Simplification:** Reduce from 4 type params (`T, A, F`) to 2 (`M, R`).

---

## Type Aliases for Convenience

```rust
// Common type aliases
pub type RealLatticeField<G, const D: usize> = LatticeGaugeField<G, D, f64, f64>;
pub type ComplexLatticeField<G, const D: usize> = LatticeGaugeField<G, D, Complex<f64>, f64>;

// Physics-specific
pub type U1Complex = ComplexLatticeField<U1, 2>;  // 2D U(1) with complex phases
pub type SU3Real = RealLatticeField<SU3, 4>;      // 4D SU(3) with real matrices
```

---

## Trait Bound Summary

### LinkVariable<G, M, R>

| Bound                   | Source               | Used For                                 |
|-------------------------|----------------------|------------------------------------------|
| `G: GaugeGroup`         | Existing             | Matrix dimension, abelian flag           |
| `M: DivisionAlgebra<R>` | `deep_causality_num` | `conjugate()`, `norm_sqr()`, `inverse()` |
| `M: Field`              | `deep_causality_num` | `+`, `-`, `*`, `/`, `one()`, `zero()`    |
| `R: RealField`          | `deep_causality_num` | `exp()`, `sin()`, `cos()`, `sqrt()`      |

### Optional Bounds

| Bound                | When Required       | Used For                           |
|----------------------|---------------------|------------------------------------|
| `M: ComplexField<R>` | U(1), complex SU(N) | `from_polar()`, `real()`, `imag()` |
| `R: From<f64>`       | Conversions         | `R::from(0.5)` for random numbers  |

---

## Migration Checklist

### Files to Modify

- [ ] `types/gauge/link_variable/mod.rs` — Add `R` param, update bounds
- [ ] `types/gauge/link_variable/ops.rs` — Fix `dagger()` for complex, update `re_trace()`
- [ ] `types/gauge/link_variable/getters.rs` — Update signatures
- [ ] `types/gauge/link_variable/display.rs` — Handle complex display
- [ ] `types/gauge/gauge_field_lattice/mod.rs` — Add `R` param, `beta: R`
- [ ] `types/gauge/gauge_field_lattice/getters.rs` — Update return types
- [ ] `types/gauge/gauge_field_lattice/ops_actions.rs` — Return `R` for action
- [ ] `types/gauge/gauge_field_lattice/ops_metropolis.rs` — Fix U(1) proposal
- [ ] `types/gauge/gauge_field_lattice/ops_monte_carlo.rs` — `ΔS: R`
- [ ] `types/gauge/gauge_field_lattice/ops_wilson.rs` — Real trace
- [ ] `types/gauge/gauge_field_lattice/ops_gauge_transform.rs` — Pass through
- [ ] `types/gauge/gauge_field_lattice/ops_gradient_flow.rs` — Update bounds
- [ ] `types/gauge/gauge_field_lattice/ops_smearing.rs` — Update bounds
- [ ] `types/gauge/gauge_field_lattice/ops_plague.rs` — Update bounds
- [ ] `types/gauge/gauge_field_lattice/ops_continuum.rs` — Update bounds
- [ ] `types/gauge/gauge_field/mod.rs` — Simplify to `<G, M, R>`
- [ ] `types/gauge/gauge_field/getters.rs` — Update return types

### Tests to Add

- [ ] `test_complex_u1_identity` — Create `LinkVariable<U1, Complex<f64>, f64>`
- [ ] `test_complex_u1_random_phase` — Generate random phases
- [ ] `test_complex_u1_metropolis` — Should thermalize to Bessel ratio
- [ ] `test_complex_plaquette_trace` — `Re[Tr(P)]` returns `f64`
- [ ] `test_su2_real_vs_complex` — Same results for real SU(2)

---

## Risks and Mitigations

| Risk                         | Impact | Mitigation                                              |
|------------------------------|--------|---------------------------------------------------------|
| Breaking API changes         | High   | Provide type aliases for common cases                   |
| Compilation time increase    | Medium | Trait bounds are static, minimal runtime cost           |
| Complex edge cases           | Medium | Extensive test coverage for both real and complex       |
| `ComplexField` not universal | Low    | Conditional compilation for M: ComplexField when needed |

---

## Success Criteria

1. **`test_thermalization_to_bessel_ratio` passes** — Metropolis converges for U1
2. **All existing tests pass** — No regressions for real f64 usage
3. **Type aliases work** — `RealLatticeField<SU3, 4>` compiles
4. **Complex U(1) works** — `LatticeGaugeField<U1, 2, Complex<f64>, f64>` compiles

---

## Appendix: Trait Hierarchy Reference

```
Field ← CommutativeRing ← Ring ← AbelianGroup
  ↓                              ↓
  └── RealField (f32, f64)       └── DivisionAlgebra<R>
  └── ComplexField<R>                  ↓
        ↓                        Complex<R>, Quaternion<R>
      Complex<R>
```

Both `f64` and `Complex<f64>` satisfy `DivisionAlgebra<f64> + Field`, making them interchangeable for matrix elements
while `f64` alone satisfies `RealField` for scalar outputs.
