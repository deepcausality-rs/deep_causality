# Lattice Gauge Field Specification

* **Product Area:** Deep Causality
* **Crate:** `deep_causality_topology`
* **Status:** Draft - Awaiting Review
* **Target:** Q1 2026
* **Classification:** Core Infrastructure Extension
* **Owner:** DeepCausality Authors

---

## 1. Executive Summary

This specification defines the **Lattice Gauge Field** infrastructure for `deep_causality_topology`. This type combines
the existing `Lattice<D>` discrete spacetime structure with gauge field theory concepts, enabling Wilson-formulation
lattice gauge theory computations.

### 1.1 Problem Statement

The current codebase has two separate but complementary structures:

| Existing Type            | Location             | Purpose                                              |
|--------------------------|----------------------|------------------------------------------------------|
| `GaugeField<G, T, A, F>` | `types/gauge_field/` | Continuum gauge fields with connection and curvature |
| `Lattice<D>`             | `types/lattice/`     | Discrete regular lattice with boundary operators     |

**Gap:** No unified type for **lattice gauge fields** where:

- Gauge degrees of freedom live on **links** (edges), not vertices
- Field strength is computed from **plaquettes** (elementary faces)
- The dynamics are governed by the **Wilson action**

### 1.2 Solution: LatticeGaugeField

| New Type                                  | Purpose                                               |
|-------------------------------------------|-------------------------------------------------------|
| `LatticeGaugeField<G, const D: usize, T>` | Gauge field on a discrete lattice with link variables |

---

## 2. Mathematical Background

> Based on David Tong's lecture notes on Lattice Gauge Theory (University of Cambridge).

### 2.1 Discretization of Spacetime

Lattice gauge theory discretizes Euclidean spacetime into a cubic, D-dimensional lattice Γ with lattice spacing $a$:

$$\Gamma = \left\{ x : x = \sum_{\mu=1}^D a n_\mu \hat{\mu} \, , \,\, n_\mu \in \mathbf{Z} \right\}$$

The lattice spacing acts as an **ultraviolet cut-off**: $\Lambda_{UV} = 1/a$.

### 2.2 Gauge Fields: Link Variables

Instead of discretizing the algebra-valued field $A_\mu$, lattice gauge theory uses **group-valued variables
** $U_\mu(x)$ living on the **links** between sites:

$$\text{link } x \rightarrow x + \hat{\mu}: \quad U_\mu(x) \in G \quad (\text{e.g., } SU(N))$$

**Relation to continuum gauge field:**

$$U_\mu(x) = e^{ia A_\mu(x)}$$

**Properties:**

- **Orientation:** $U_{-\mu}(x + \hat{\mu}) = U_\mu(x)^{-1} = U_\mu(x)^\dagger$ (for unitary groups)
- For small $a$: $U_\mu(x) \approx 1 + ia A_\mu(x) + O(a^2)$

**Gauge Transformation:**

$$U_\mu(x) \rightarrow \Omega(x) U_\mu(x) \Omega^\dagger(x + \hat{\mu})$$

where $\Omega(x) \in G$ is a local gauge transformation at each site.

### 2.3 The Wilson Loop (Plaquette)

The simplest **gauge-invariant** object is the trace of the product of link variables around a single square (*
*plaquette**) in the $\mu$-$\nu$ plane:

$$W_{\square} = \text{tr } U_\mu(x) U_\nu(x + \hat{\mu}) U_\mu^\dagger(x + \hat{\nu}) U_\nu^\dagger(x)$$

The plaquette is the discrete analogue of the field strength tensor $F_{\mu\nu}$.

### 2.4 The Wilson Action

The **Wilson action** is defined as:

$$S_{\text{Wilson}} = -\frac{\beta}{2N} \sum_{\square} \left( W_{\square} + W_{\square}^\dagger \right)$$

where the coupling $\beta$ is related to the continuum coupling $g$ by:

$$\frac{\beta}{2N} = \frac{1}{g^2}$$

Equivalently:

$$S_W = \beta \sum_p \left(1 - \frac{1}{N} \text{Re}[\text{Tr}(U_p)]\right)$$

**Continuum Limit:** As $a \to 0$, the Wilson action reproduces the Yang-Mills action:

$$S_{\text{Wilson}} = \frac{1}{2g^2} \int d^4x \, \text{tr } F_{\mu\nu} F^{\mu\nu} + O(a^2)$$

### 2.5 The Haar Measure

To preserve gauge invariance in the path integral, integration over link variables uses the **Haar measure**:

$$Z = \int \prod_{(x, \mu)} dU_\mu(x) \, e^{-S_{\text{Wilson}}}$$

**Properties of the Haar measure:**

- **Invariance:** $\int dU \, f(U) = \int dU \, f(\Omega U) = \int dU \, f(U \Omega)$
- **Normalization:** $\int dU \, 1 = 1$
- **Orthogonality:** $\int dU \, U_{ij} U_{kl}^\dagger = \frac{1}{N} \delta_{il} \delta_{jk}$

### 2.6 Elitzur's Theorem

> [!IMPORTANT]
> For any **non-gauge-invariant** operator $\mathcal{O}$:
> $$\langle \mathcal{O} \rangle = 0$$

This means only gauge-invariant quantities (Wilson loops, traces) have non-zero expectation values.

### 2.7 Strong Coupling Expansion & Confinement

In the limit $\beta \ll 1$ (strong coupling), the expectation value of a large rectangular Wilson loop $W[C]$ with
area $A$ exhibits an **area law** (confinement):

$$\langle W[C] \rangle \sim \left( \frac{\beta}{2N^2} \right)^{A/a^2} = e^{-\sigma A}$$

where the **string tension** $\sigma$ is:

$$\sigma = -\frac{1}{a^2} \log \left( \frac{\beta}{2N^2} \right)$$

This demonstrates quark confinement in lattice QCD.

### 2.8 General Wilson Loops

A **Wilson loop** $W(C)$ for an arbitrary closed path $C$ is:

$$W(C) = \text{Tr}\left[\prod_{l \in C} U_l\right]$$

**Applications:**

- **Static quark potential:** Extract from $R \times T$ rectangular Wilson loops
- **Confinement order parameter:** Area law vs perimeter law
- **Topological observables:** Polyakov loops for finite temperature

### 2.9 Relationship to Continuum GaugeField

| Concept            | Continuum `GaugeField<G>`                                               | Lattice `LatticeGaugeField<G>`                          |
|--------------------|-------------------------------------------------------------------------|---------------------------------------------------------|
| Gauge DOF location | Connection tensor $A_\mu$ at points                                     | Link variables $U_\mu(x)$ on edges                      |
| Field strength     | $F_{\mu\nu} = \partial_\mu A_\nu - \partial_\nu A_\mu + [A_\mu, A_\nu]$ | Plaquette $W_{\square}$                                 |
| Action             | $S = \frac{1}{2g^2}\int \text{tr } F_{\mu\nu} F^{\mu\nu}$               | Wilson action $S_{\text{Wilson}}$                       |
| Gauge transform    | $A \to \Omega A \Omega^{-1} + \Omega \partial \Omega^{-1}$              | $U_\mu(x) \to \Omega(x) U_\mu(x) \Omega^\dagger(x+\mu)$ |
| Path integral      | Ill-defined without regularization                                      | Well-defined with Haar measure                          |

---

## 3. Architecture Decision: Option B (Integrated Module)

> [!IMPORTANT]
> **Recommendation: Option B** — Integrate into existing `gauge_field` module.

### 3.1 Rationale

| Criterion          | Option A (Separate Module)         | Option B (Integrated)                       |
|--------------------|------------------------------------|---------------------------------------------|
| **Code reuse**     | Duplicate `GaugeGroup` trait usage | ✅ Share `GaugeGroup`, groups (U1, SU2, SU3) |
| **Cohesion**       | Separate conceptual domains        | ✅ Unified gauge field family                |
| **HKT extensions** | New extension file                 | ✅ Add to `hkt_gauge_field/`                 |
| **User discovery** | Two module paths                   | ✅ Single `gauge_field` import path          |
| **Maintenance**    | Two sets of gauge group impls      | ✅ Single source of truth                    |

### 3.2 Key Insight

`LatticeGaugeField<G, D, T>` **reuses** the `GaugeGroup` trait completely:

- `G::LIE_ALGEBRA_DIM` → size of link variable matrices
- `G::IS_ABELIAN` → simplifies plaquette computation (commutative product)
- `G::structure_constant(a, b, c)` → needed for SU(N) group multiplication

This makes Option B the clear winner for maintainability and consistency.

### 3.3 Reuse of Existing Differential Operators

The existing `Manifold` type provides differential operators in `src/types/manifold/differential/`:

| Operator            | Method                   | Description            |
|---------------------|--------------------------|------------------------|
| Exterior derivative | `exterior_derivative(k)` | d: k-form → (k+1)-form |
| Codifferential      | `codifferential(k)`      | δ: k-form → (k-1)-form |
| Hodge star          | `hodge_star(k)`          | ⋆: k-form → (n-k)-form |
| Laplacian           | `laplacian(k)`           | Δ = dδ + δd            |

#### 3.3.1 What CAN Be Reused

| Component                           | Source                  | Reason                                                                  |
|-------------------------------------|-------------------------|-------------------------------------------------------------------------|
| **`Lattice<D>` boundary operators** | `CWComplex` trait       | `Lattice` already implements `boundary_matrix(k)` returning `CsrMatrix` |
| **`CsrMatrix<i8>` sparse algebra**  | `deep_causality_sparse` | Same machinery for boundary/coboundary                                  |
| **`LatticeCell<D>` iteration**      | `types/lattice/`        | Cell enumeration for links/plaquettes                                   |

#### 3.3.2 What CANNOT Be Directly Reused

| Component              | In `Manifold`                   | In `LatticeGaugeField`                    | Reason                                |
|------------------------|---------------------------------|-------------------------------------------|---------------------------------------|
| **Degrees of freedom** | Scalar/vector k-forms           | **Group-valued** link variables $U \in G$ | Group multiplication ≠ linear algebra |
| **Field strength**     | $F = d A$ (exterior derivative) | Plaquette $U_{\mu\nu}$ (group product)    | Non-linear on non-abelian groups      |
| **Hodge star**         | Metric-dependent                | Not applicable                            | No Riemannian structure on link space |

> [!NOTE]
> The `Manifold` differential operators act on **scalar-valued differential forms** over a simplicial complex.
> Lattice gauge fields have **Lie group-valued** degrees of freedom on edges, requiring **group multiplication**
> rather than linear operations. The plaquette $U_{\mu\nu} = U_\mu U_\nu U_\mu^\dagger U_\nu^\dagger$ is fundamentally
> non-linear for non-abelian groups.

#### 3.3.3 Lattice Boundary Operator Reuse

The existing `Lattice<D>` implements `CWComplex` trait which provides:

```rust
// Already implemented in types/lattice/mod.rs
impl<const D: usize> CWComplex for Lattice<D> {
    fn boundary_matrix(&self, k: usize) -> CsrMatrix<i8>;
    fn num_cells(&self, k: usize) -> usize;
    fn cells(&self, k: usize) -> Box<dyn Iterator<Item=LatticeCell<D>> + '_>;
}
```

`LatticeGaugeField` will use these boundary operators for:

- **Enumerating edges** (1-cells) to index link variables
- **Enumerating faces** (2-cells) to identify plaquettes
- **Computing staples** by iterating coboundary of an edge

---

## 4. File Structure

### 4.1 New Files to Create

| File Path                                             | Description                         |
|-------------------------------------------------------|-------------------------------------|
| `src/types/gauge_field/lattice_gauge_field.rs`        | `LatticeGaugeField<G, D, T>` struct |
| `src/types/gauge_field/link_variable.rs`              | `LinkVariable<G, T>` wrapper type   |
| `src/extensions/hkt_gauge_field/hkt_lattice_gauge.rs` | HKT witness for lattice gauge field |

### 4.2 Files to Modify

| File Path                               | Changes                                                   |
|-----------------------------------------|-----------------------------------------------------------|
| `src/types/gauge_field/mod.rs`          | Add `pub mod lattice_gauge_field; pub mod link_variable;` |
| `src/extensions/hkt_gauge_field/mod.rs` | Add `pub mod hkt_lattice_gauge;`                          |
| `src/lib.rs`                            | Re-export `LatticeGaugeField`, `LinkVariable`             |

---

## 5. Type Specification

### 5.1 LinkVariable<G, T>

Wrapper for a group element representing a gauge link.

```rust
// Location: src/types/gauge_field/link_variable.rs

use crate::GaugeGroup;
use deep_causality_tensor::CausalTensor;
use std::marker::PhantomData;

/// A link variable U_μ(n) ∈ G on a lattice edge.
///
/// For SU(N), this is an N×N unitary matrix with det = 1.
/// Stored as a flattened tensor of shape [N, N] (complex entries
/// require 2 real components per element or use num::Complex).
#[derive(Debug, Clone)]
pub struct LinkVariable<G: GaugeGroup, T> {
    /// Matrix elements of the group element.
    /// For SU(N): N² complex components.
    data: CausalTensor<T>,
    _gauge: PhantomData<G>,
}

impl<G: GaugeGroup, T: Clone + Default> LinkVariable<G, T> {
    // --- Constructors ---

    /// Create the identity link (unit element of G).
    pub fn identity() -> Self;

    /// Create from raw matrix data.
    pub fn from_matrix(data: CausalTensor<T>) -> Self;

    /// Create a random link (for Monte Carlo initialization).
    /// Requires T: From<f64> to generate random values.
    pub fn random() -> Self where
        T: From<f64>;

    // --- Getters ---

    /// Matrix data as tensor.
    pub fn matrix(&self) -> &CausalTensor<T>;

    /// Lie algebra dimension (N² - 1 for SU(N)).
    pub fn lie_dim() -> usize { G::LIE_ALGEBRA_DIM }

    // --- Operations ---

    /// Hermitian conjugate U†.
    pub fn dagger(&self) -> Self where
        T: Clone + std::ops::Neg<Output=T>;

    /// Group multiplication: self * other.
    pub fn mul(&self, other: &Self) -> Self where
        T: Clone + std::ops::Mul<Output=T> + std::ops::Add<Output=T>;

    /// Trace of the matrix.
    pub fn trace(&self) -> T where
        T: Clone + std::ops::Add<Output=T>;

    /// Real part of trace (for action computation).
    pub fn re_trace(&self) -> T where
        T: Clone;
}
```

### 5.2 LatticeGaugeField<G, D, T>

The main lattice gauge field type.

```rust
// Location: src/types/gauge_field/lattice_gauge_field.rs

use crate::{GaugeGroup, Lattice, LatticeCell};
use crate::gauge_field::LinkVariable;
use std::collections::HashMap;
use std::sync::Arc;

/// A gauge field on a D-dimensional lattice.
///
/// Link variables U_μ(n) are stored on each edge of the lattice.
/// For a hypercubic lattice: D * num_vertices links total.
///
/// # Type Parameters
/// * `G` - Gauge group (U1, SU2, SU3, etc.)
/// * `D` - Spacetime dimension
/// * `T` - Scalar type for matrix elements
#[derive(Debug, Clone)]
pub struct LatticeGaugeField<G: GaugeGroup, const D: usize, T> {
    /// The underlying lattice structure.
    lattice: Arc<Lattice<D>>,

    /// Link variables indexed by LatticeCell (1-cells only).
    /// Key: edge cell, Value: group element
    links: HashMap<LatticeCell<D>, LinkVariable<G, T>>,

    /// Coupling constant g (or β = 2N/g² cached).
    beta: T,
}

impl<G: GaugeGroup, const D: usize, T: Clone + Default> LatticeGaugeField<G, D, T> {
    // --- Constructors ---

    /// Create with all links set to identity.
    pub fn identity(lattice: Arc<Lattice<D>>, beta: T) -> Self;

    /// Create with random links (hot start for Monte Carlo).
    pub fn random(lattice: Arc<Lattice<D>>, beta: T) -> Self
    where
        T: From<f64>;

    /// Create from explicit link data.
    pub fn from_links(
        lattice: Arc<Lattice<D>>,
        links: HashMap<LatticeCell<D>, LinkVariable<G, T>>,
        beta: T,
    ) -> Self;

    // --- Getters ---

    /// The underlying lattice.
    pub fn lattice(&self) -> &Lattice<D>;

    /// Coupling parameter β = 2N/g².
    pub fn beta(&self) -> &T;

    /// Number of links (edges).
    pub fn num_links(&self) -> usize;

    /// Get link variable for an edge.
    pub fn link(&self, edge: &LatticeCell<D>) -> Option<&LinkVariable<G, T>>;

    /// Mutable access to a link (for Monte Carlo updates).
    pub fn link_mut(&mut self, edge: &LatticeCell<D>) -> Option<&mut LinkVariable<G, T>>;

    // --- Plaquette Operations ---

    /// Compute the plaquette U_μν(n) at a given site in a given plane.
    /// Returns the ordered product of 4 links around the elementary square.
    pub fn plaquette(&self, site: &[usize; D], mu: usize, nu: usize) -> LinkVariable<G, T>
    where
        T: Clone + std::ops::Mul<Output=T> + std::ops::Add<Output=T>;

    /// Average plaquette value: (1/N_p) Σ_p Re[Tr(U_p)] / N.
    /// This is related to the action density.
    pub fn average_plaquette(&self) -> T
    where
        T: Clone + std::ops::Mul<Output=T> + std::ops::Add<Output=T> + std::ops::Div<Output=T> + From<f64>;

    // --- Action ---

    /// Compute the Wilson action: S = β Σ_p (1 - Re[Tr(U_p)]/N).
    pub fn wilson_action(&self) -> T
    where
        T: Clone + std::ops::Mul<Output=T> + std::ops::Add<Output=T> + std::ops::Sub<Output=T> + From<f64>;

    /// Action contribution from a single plaquette.
    pub fn plaquette_action(&self, site: &[usize; D], mu: usize, nu: usize) -> T
    where
        T: Clone + std::ops::Mul<Output=T> + std::ops::Add<Output=T> + std::ops::Sub<Output=T> + From<f64>;

    // --- Wilson Loops ---

    /// Compute an R×T rectangular Wilson loop.
    /// Used for extracting the static quark potential.
    pub fn wilson_loop(&self, corner: &[usize; D], r_dir: usize, t_dir: usize, r: usize, t: usize) -> T
    where
        T: Clone + std::ops::Mul<Output=T> + std::ops::Add<Output=T>;

    /// Polyakov loop (temporal Wilson line wrapping the lattice).
    /// Order parameter for confinement/deconfinement.
    pub fn polyakov_loop(&self, spatial_site: &[usize; D]) -> T
    where
        T: Clone + std::ops::Mul<Output=T> + std::ops::Add<Output=T>;

    // --- Gauge Transformations ---

    /// Apply a gauge transformation: U_μ(n) → g(n) U_μ(n) g(n+μ)†.
    /// `gauge_fn` provides g(n) for each lattice site.
    pub fn gauge_transform<F>(&mut self, gauge_fn: F)
    where
        F: Fn(&[usize; D]) -> LinkVariable<G, T>,
        T: Clone + std::ops::Mul<Output=T> + std::ops::Add<Output=T>;

    // --- Continuum Limit ---

    /// Extract the field strength tensor F_μν at a site (naive discretization).
    /// F_μν ≈ (1/ia²)(U_μν - U_μν†) / 2 for small a.
    pub fn field_strength(&self, site: &[usize; D], mu: usize, nu: usize) -> CausalTensor<T>
    where
        T: Clone + std::ops::Sub<Output=T> + std::ops::Mul<Output=T>;

    /// Topological charge density q(n) from plaquettes.
    /// Q = Σ_n q(n) is an integer for smooth configurations.
    pub fn topological_charge_density(&self, site: &[usize; D]) -> T
    where
        T: Clone + std::ops::Mul<Output=T> + std::ops::Add<Output=T> + std::ops::Sub<Output=T> + From<f64>;
}
```

---

## 6. HKT Extension

### 6.1 LatticeGaugeFieldWitness

```rust
// Location: src/extensions/hkt_gauge_field/hkt_lattice_gauge.rs

use crate::{GaugeGroup, LatticeGaugeField, Lattice};
use deep_causality_haft::{HKT, Functor, Monad, NoConstraint, Satisfies};
use std::sync::Arc;

/// HKT witness for LatticeGaugeField.
///
/// Enables functional transformations of lattice gauge fields.
pub struct LatticeGaugeFieldWitness<G: GaugeGroup, const D: usize>;

impl<G: GaugeGroup, const D: usize> HKT for LatticeGaugeFieldWitness<G, D> {
    type Constraint = NoConstraint;
    type Type<T> = LatticeGaugeField<G, D, T>
    where
        T: Satisfies<NoConstraint> + Clone + Default;
}

/// Functor implementation: map over scalar type.
impl<G: GaugeGroup, const D: usize> Functor<LatticeGaugeFieldWitness<G, D>>
for LatticeGaugeFieldWitness<G, D>
{
    fn fmap<A, B, F>(fa: LatticeGaugeField<G, D, A>, f: F) -> LatticeGaugeField<G, D, B>
    where
        A: Satisfies<NoConstraint> + Clone + Default,
        B: Satisfies<NoConstraint> + Clone + Default,
        F: FnMut(A) -> B,
    {
        // Transform all link variables element-wise
        todo!()
    }
}
```

### 6.2 Applicative for Monte Carlo Updates

```rust
/// Applicative enables combining independent link updates.
impl<G: GaugeGroup, const D: usize> Applicative<LatticeGaugeFieldWitness<G, D>>
for LatticeGaugeFieldWitness<G, D>
{
    fn pure<A>(a: A) -> LatticeGaugeField<G, D, A>
    where
        A: Satisfies<NoConstraint> + Clone + Default,
    {
        // Constant field (all links have same embedded value)
        todo!()
    }

    fn apply<A, B, F>(
        ff: LatticeGaugeField<G, D, F>,
        fa: LatticeGaugeField<G, D, A>,
    ) -> LatticeGaugeField<G, D, B>
    where
        F: FnMut(A) -> B,
    {
        // Apply link-by-link
        todo!()
    }
}
```

---

## 7. Integration with Existing Types

### 7.1 GaugeGroup Reuse

`LatticeGaugeField<G, D, T>` uses the **same** `GaugeGroup` implementations:

```rust
use deep_causality_topology::{U1, SU2, SU3, LatticeGaugeField, Lattice};

// Lattice QED: U(1) gauge field on 4D lattice
type LatticeQED = LatticeGaugeField<U1, 4, f64>;

// Lattice QCD: SU(3) gauge field on 4D lattice  
type LatticeQCD = LatticeGaugeField<SU3, 4, f64>;

// Weak isospin: SU(2) gauge field
type LatticeWeak = LatticeGaugeField<SU2, 4, f64>;
```

### 7.2 Lattice Integration

Leverages existing `Lattice<D>` infrastructure:

```rust
let lattice = Arc::new(Lattice::hypercubic_torus(16));  // 16^4 lattice
let qcd = LatticeGaugeField::<SU3, 4, f64>::random(lattice.clone(), 6.0);

// Use existing lattice methods
assert_eq!(lattice.num_cells(1), qcd.num_links());  // edges = links
```

### 7.3 Relationship to Continuum GaugeField

| Operation   | Continuum `GaugeField<G, T, A, F>` | Lattice `LatticeGaugeField<G, D, T>`          |
|-------------|------------------------------------|-----------------------------------------------|
| Creation    | `GaugeField::new(manifold, ...)`   | `LatticeGaugeField::identity(lattice, β)`     |
| Curvature   | `field.field_strength()` → tensor  | `field.plaquette(site, μ, ν)` → group element |
| Action      | Physics crate responsibility       | `field.wilson_action()`                       |
| HKT witness | `GaugeFieldWitness<T>`             | `LatticeGaugeFieldWitness<G, D>`              |

---

## 8. Verification Plan

### 8.1 Automated Tests

| Test Category               | Test Description                            | Command                                               |
|-----------------------------|---------------------------------------------|-------------------------------------------------------|
| **Identity plaquette**      | `plaquette()` of identity config = identity | `cargo test -p deep_causality_topology lattice_gauge` |
| **Action = 0 for identity** | `wilson_action()` of identity config = 0    | `cargo test -p deep_causality_topology lattice_gauge` |
| **Gauge invariance**        | Action unchanged under `gauge_transform()`  | `cargo test -p deep_causality_topology lattice_gauge` |
| **Plaquette normalization** | Average plaquette ∈ [0, 1]                  | `cargo test -p deep_causality_topology lattice_gauge` |
| **Abelian simplification**  | U(1) plaquette is commutative product       | `cargo test -p deep_causality_topology lattice_gauge` |
| **Link count**              | `num_links()` = D × `lattice.num_cells(0)`  | `cargo test -p deep_causality_topology lattice_gauge` |

### 8.2 Existing Test Infrastructure

Tests should be added to:

```
deep_causality_topology/tests/types/gauge_field/lattice_gauge_field_tests.rs
deep_causality_topology/tests/extensions/hkt_lattice_gauge_tests.rs
```

Run all topology tests:

```bash
cargo test -p deep_causality_topology --all-features
```

---

## 9. Example Usage

```rust
use deep_causality_topology::{
    Lattice, LatticeGaugeField, SU3,
};
use std::sync::Arc;

fn main() {
    // Create a 4D toroidal lattice (periodic boundaries)
    let lattice = Arc::new(Lattice::hypercubic_torus(8));  // 8^4 = 4096 sites

    // Initialize SU(3) gauge field at β = 6.0 (typical QCD coupling)
    let beta = 6.0_f64;
    let mut qcd = LatticeGaugeField::<SU3, 4, f64>::random(lattice, beta);

    // Compute Wilson action
    let action = qcd.wilson_action();
    println!("Initial action: {}", action);

    // Compute average plaquette
    let avg_plaq = qcd.average_plaquette();
    println!("Average plaquette: {}", avg_plaq);

    // Measure a 3x3 Wilson loop (static quark potential)
    let corner = [0, 0, 0, 0];
    let w_loop = qcd.wilson_loop(&corner, 0, 3, 3, 3);
    println!("3x3 Wilson loop: {}", w_loop);
}
```

---

## 10. Monte Carlo Updates

Monte Carlo methods enable importance sampling of gauge configurations according to the Boltzmann weight $e^{-S}$.

### 10.1 Mathematical Background

The goal is to generate configurations $\{U\}$ distributed according to:

$$P[U] = \frac{1}{Z} e^{-S[U]}$$

This is achieved via Markov chain Monte Carlo (MCMC) with detailed balance:

$$P[U] T(U \to U') = P[U'] T(U' \to U)$$

### 10.2 Metropolis Algorithm

For a proposed link update $U_\mu(x) \to U'_\mu(x)$:

1. Compute action change: $\Delta S = S[U'] - S[U]$
2. Accept with probability: $P_{\text{accept}} = \min(1, e^{-\Delta S})$

```rust
impl<G: GaugeGroup, const D: usize, T> LatticeGaugeField<G, D, T> {
    /// Perform a single Metropolis update on a link.
    ///
    /// # Arguments
    /// * `edge` - The link to update
    /// * `proposal` - Function generating a random proposal U'
    /// * `rng` - Random number generator (from deep_causality_rand)
    ///
    /// # Returns
    /// `true` if the update was accepted, `false` otherwise.
    pub fn metropolis_update<R, F>(
        &mut self,
        edge: &LatticeCell<D>,
        proposal: F,
        rng: &mut R,
    ) -> bool
    where
        R: deep_causality_rand::Rng,
        F: Fn(&LinkVariable<G, T>, &mut R) -> LinkVariable<G, T>,
        T: Clone + std::ops::Sub<Output=T> + PartialOrd + From<f64>;

    /// Perform a full Metropolis sweep over all links.
    pub fn metropolis_sweep<R, F>(&mut self, proposal: F, rng: &mut R) -> f64
    where
        R: deep_causality_rand::Rng,
        F: Fn(&LinkVariable<G, T>, &mut R) -> LinkVariable<G, T>,
        T: Clone + std::ops::Sub<Output=T> + PartialOrd + From<f64>;
}
```

### 10.3 Heat Bath Algorithm

For SU(2), the Creutz heat bath directly samples from the local Boltzmann distribution:

$$P(U_\mu(x)) \propto e^{-S_{\text{local}}(U_\mu(x))}$$

where the local action depends only on the **staple** $V$:

$$V = \sum_{\nu \neq \mu} \left[ U_\nu(x+\hat{\mu}) U_\mu^\dagger(x+\hat{\nu}) U_\nu^\dagger(x) + U_\nu^\dagger(x+\hat{\mu}-\hat{\nu}) U_\mu^\dagger(x-\hat{\nu}) U_\nu(x-\hat{\nu}) \right]$$

```rust
impl<G: GaugeGroup, const D: usize, T> LatticeGaugeField<G, D, T> {
    /// Compute the staple sum for a given link.
    pub fn staple(&self, edge: &LatticeCell<D>) -> LinkVariable<G, T>
    where
        T: Clone + std::ops::Mul<Output=T> + std::ops::Add<Output=T>;

    /// Perform a heat bath update on a single link (SU(2) subgroup method).
    pub fn heat_bath_update<R>(&mut self, edge: &LatticeCell<D>, rng: &mut R)
    where
        R: deep_causality_rand::Rng,
        T: Clone + std::ops::Mul<Output=T> + std::ops::Add<Output=T> + From<f64>;

    /// Perform a full heat bath sweep using Cabibbo-Marinari for SU(N).
    pub fn heat_bath_sweep<R>(&mut self, rng: &mut R) -> f64
    where
        R: deep_causality_rand::Rng,
        T: Clone + std::ops::Mul<Output=T> + std::ops::Add<Output=T> + From<f64>;
}
```

### 10.4 Overrelaxation

Deterministic updates that preserve energy but decorrelate configurations:

$$U_\mu(x) \to V^\dagger U_\mu(x)^{-1} V^\dagger$$

```rust
impl<G: GaugeGroup, const D: usize, T> LatticeGaugeField<G, D, T> {
    /// Perform an overrelaxation update on a single link.
    pub fn overrelaxation_update(&mut self, edge: &LatticeCell<D>)
    where
        T: Clone + std::ops::Mul<Output=T> + std::ops::Add<Output=T>;

    /// Perform a full overrelaxation sweep.
    pub fn overrelaxation_sweep(&mut self)
    where
        T: Clone + std::ops::Mul<Output=T> + std::ops::Add<Output=T>;
}
```

---

## 11. Improved Actions

The Wilson action has $O(a^2)$ discretization errors. Improved actions reduce these errors.

### 11.1 Symanzik Improvement

The **Symanzik-improved action** includes both plaquette and rectangle terms:

$$S_{\text{Symanzik}} = \beta \left[ c_0 \sum_{\square} \left(1 - \frac{1}{N} \text{Re Tr } U_\square \right) + c_1 \sum_{\boxminus} \left(1 - \frac{1}{N} \text{Re Tr } U_{\boxminus} \right) \right]$$

where $U_{\boxminus}$ is a $1 \times 2$ rectangle and $c_0 + 8 c_1 = 1$ with $c_1 = -1/12$ for tree-level improvement.

### 11.2 Iwasaki Action

$$c_0 = 1 - 8 c_1, \quad c_1 = -0.331$$

Optimized for reducing short-distance lattice artifacts.

### 11.3 DBW2 Action

$$c_0 = 1 - 8 c_1, \quad c_1 = -1.4088$$

Aggressive improvement targeting instanton physics.

```rust
/// Improved action coefficients.
#[derive(Debug, Clone, Copy)]
pub struct ImprovedActionCoeffs<T> {
    /// Plaquette coefficient c_0.
    pub c0: T,
    /// Rectangle coefficient c_1.
    pub c1: T,
}

impl<T: From<f64> + Clone> ImprovedActionCoeffs<T> {
    /// Tree-level Symanzik: c_1 = -1/12.
    pub fn symanzik() -> Self;

    /// Iwasaki: c_1 = -0.331.
    pub fn iwasaki() -> Self;

    /// DBW2: c_1 = -1.4088.
    pub fn dbw2() -> Self;

    /// Custom coefficients.
    pub fn custom(c0: T, c1: T) -> Self;
}

impl<G: GaugeGroup, const D: usize, T> LatticeGaugeField<G, D, T> {
    /// Compute a 1×2 rectangle Wilson loop at a site.
    pub fn rectangle(&self, site: &[usize; D], mu: usize, nu: usize) -> LinkVariable<G, T>
    where
        T: Clone + std::ops::Mul<Output=T> + std::ops::Add<Output=T>;

    /// Compute the Symanzik-improved action with given coefficients.
    pub fn improved_action(&self, coeffs: &ImprovedActionCoeffs<T>) -> T
    where
        T: Clone + std::ops::Mul<Output=T> + std::ops::Add<Output=T> + std::ops::Sub<Output=T> + From<f64>;
}
```

---

## 12. Smearing Algorithms

Smearing reduces UV fluctuations while preserving long-distance physics, useful for:

- Reducing statistical noise in observables
- Improving overlap with ground states
- Constructing smooth gauge-fixed configurations

### 12.1 APE Smearing

Replace spatial links with a weighted average including staples:

$$U_\mu^{\text{APE}}(x) = \text{Proj}_{SU(N)} \left[ (1 - \alpha) U_\mu(x) + \frac{\alpha}{6} V_\mu(x) \right]$$

where $V_\mu$ is the staple sum and $\text{Proj}_{SU(N)}$ projects back to the group.

### 12.2 HYP Smearing

**Hypercubic** smearing applies APE-like steps in a hierarchical pattern:

1. Smear in 2D sub-hypercubes (decorated staples)
2. Smear in 3D sub-hypercubes using step 1 links
3. Smear in 4D using step 2 links

Three parameters: $(\alpha_1, \alpha_2, \alpha_3)$ with typical values $(0.75, 0.6, 0.3)$.

### 12.3 Stout Smearing

Analytic, differentiable smearing:

$$U_\mu^{\text{stout}}(x) = e^{i Q_\mu(x)} U_\mu(x)$$

where $Q_\mu(x) = \frac{\rho}{2i}\left[ \Omega_\mu(x) - \Omega_\mu^\dagger(x) - \frac{1}{N} \text{tr}(\Omega_\mu - \Omega_\mu^\dagger) \right]$

and $\Omega_\mu = V_\mu U_\mu^\dagger$.

```rust
/// Smearing parameters.
#[derive(Debug, Clone)]
pub struct SmearingParams<T> {
    /// Smearing weight α or ρ.
    pub alpha: T,
    /// Number of smearing iterations.
    pub n_steps: usize,
}

impl<G: GaugeGroup, const D: usize, T> LatticeGaugeField<G, D, T> {
    /// Perform APE smearing on spatial links.
    pub fn ape_smear(&mut self, params: &SmearingParams<T>)
    where
        T: Clone + std::ops::Mul<Output=T> + std::ops::Add<Output=T> + std::ops::Div<Output=T> + From<f64>;

    /// Perform HYP smearing.
    pub fn hyp_smear(&mut self, alpha1: T, alpha2: T, alpha3: T)
    where
        T: Clone + std::ops::Mul<Output=T> + std::ops::Add<Output=T> + From<f64>;

    /// Perform stout smearing (n_steps iterations).
    pub fn stout_smear(&mut self, params: &SmearingParams<T>)
    where
        T: Clone + std::ops::Mul<Output=T> + std::ops::Add<Output=T> + std::ops::Sub<Output=T> + From<f64>;

    /// Project a matrix to SU(N) (needed for APE/HYP).
    fn project_sun(matrix: &CausalTensor<T>) -> LinkVariable<G, T>
    where
        T: Clone + std::ops::Mul<Output=T> + std::ops::Add<Output=T> + From<f64>;
}
```

---

## 13. Gradient Flow (Wilson Flow)

Gradient flow provides a well-defined renormalization procedure by smoothing configurations along a fictitious "flow
time" $t$.

### 13.1 Flow Equation

$$\frac{d U_\mu(x, t)}{dt} = -g_0^2 \left[ \partial_{x,\mu} S[U(t)] \right] U_\mu(x, t)$$

where the Lie derivative of the action generates the flow.

For the Wilson action, using Runge-Kutta integration:

$$U_\mu(x, t + \epsilon) = e^{\epsilon Z_\mu(x, t)} U_\mu(x, t)$$

where $Z_\mu$ is the traceless anti-Hermitian part of the staple.

### 13.2 Observables at Flow Time $t$

At flow time $t$, observables are automatically renormalized at scale $\mu \sim 1/\sqrt{8t}$:

- **$t^2 \langle E(t) \rangle$**: Scale-setting quantity ($t_0$ or $w_0$)
- **$\langle t^2 E(t) \rangle = 0.3$**: Defines reference scale $t_0$
- **$t \frac{d}{dt} t^2 \langle E(t) \rangle |_{t=w_0^2} = 0.3$**: Defines $w_0$

where $E(t)$ is the action density at flow time $t$.

### 13.3 Integration Methods

- **Euler:** $O(\epsilon)$ errors, simple but slow
- **Runge-Kutta 3rd order:** Standard choice, $O(\epsilon^3)$ errors
- **Adaptive step size:** Adjust $\epsilon$ based on local truncation error

```rust
/// Gradient flow parameters.
#[derive(Debug, Clone)]
pub struct FlowParams<T> {
    /// Flow time step ε.
    pub epsilon: T,
    /// Target flow time t.
    pub t_max: T,
    /// Integration method.
    pub method: FlowMethod,
}

#[derive(Debug, Clone, Copy)]
pub enum FlowMethod {
    /// Simple Euler integration.
    Euler,
    /// 3rd order Runge-Kutta (recommended).
    RungeKutta3,
    /// Adaptive step size RK.
    AdaptiveRK,
}

impl<G: GaugeGroup, const D: usize, T> LatticeGaugeField<G, D, T> {
    /// Perform a single gradient flow step.
    pub fn flow_step(&mut self, epsilon: T)
    where
        T: Clone + std::ops::Mul<Output=T> + std::ops::Add<Output=T> + std::ops::Sub<Output=T> + From<f64>;

    /// Flow to target time t using specified method.
    pub fn flow_to(&mut self, params: &FlowParams<T>) -> Vec<(T, T)>
    where
        T: Clone + std::ops::Mul<Output=T> + std::ops::Add<Output=T> + std::ops::Sub<Output=T> + std::ops::Div<Output=T> + From<f64> + PartialOrd;

    /// Compute the clover-discretized energy density E(t).
    pub fn energy_density(&self) -> T
    where
        T: Clone + std::ops::Mul<Output=T> + std::ops::Add<Output=T> + std::ops::Div<Output=T> + From<f64>;

    /// Compute t² E(t) for scale setting.
    pub fn t2_energy(&self, t: T) -> T
    where
        T: Clone + std::ops::Mul<Output=T> + std::ops::Add<Output=T> + std::ops::Div<Output=T> + From<f64>;

    /// Find t_0 scale (where t² E(t) = 0.3).
    pub fn find_t0(&mut self, params: &FlowParams<T>) -> T
    where
        T: Clone + std::ops::Mul<Output=T> + std::ops::Add<Output=T> + std::ops::Sub<Output=T> + std::ops::Div<Output=T> + From<f64> + PartialOrd;
}
```

> [!NOTE]
> GPU acceleration is **not** listed as a separate feature — it is provided transparently by the underlying
> data structures (`CausalTensor`, `CausalMultiVector`, `CsrMatrix`) which support backend-agnostic computation.

---

## 14. References

1. Wilson, K. G. (1974). "Confinement of quarks". Phys. Rev. D 10, 2445.
2. Creutz, M. (1983). "Quarks, Gluons and Lattices". Cambridge University Press.
3. Rothe, H. J. (2005). "Lattice Gauge Theories: An Introduction". World Scientific.
4. Tong, D. (2018). "Gauge Theory". University of Cambridge Lecture Notes. Section 4: Lattice Gauge Theory.
5. Lüscher, M. (2010). "Properties and uses of the Wilson flow in lattice QCD". JHEP 08, 071.
6. Morningstar, C. & Peardon, M. (2004). "Analytic smearing of SU(3) link variables". Phys. Rev. D 69, 054501.
