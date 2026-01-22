# Gauge-Source Unification: First-Class Source Support in LGT

**Author:** AI Assistant
**Date:** 2026-01-22
**Status:** Draft
**Version:** 1.0

---

## 1. Executive Summary

This specification outlines the architectural expansion of the `LatticeGaugeField` type to include a generic **Source
Field** ($S$). This change transforms the library from a "Pure Gauge Simulator" (Vacuum only) into a "General Gauge
Framework" capable of modeling fields interacting with external sources—whether dynamic matter (QCD fermions), static
particles, or empirical data (Inverse Metrology).

The expansion introduces `LatticeGaugeField<..., S = ()>`, ensuring backward compatibility for existing vacuum
simulations while enabling new capabilities for Chrono-Gauge and Kinetic-Gauge theories.

## 2. Problem Statement

### 2.1 Current Limitation: Vacuum-Centric Design

The current definition of `LatticeGaugeField` encapsulates only the **Gauge Link Variables** ($U_\mu$):

```rust
pub struct LatticeGaugeField<G, const D: usize, M, R> {
    lattice: Arc<Lattice<D>>,
    links: HashMap<LatticeCell<D>, LinkVariable<G, M, R>>,
    beta: R,
}
```

This represents a field in **Vacuum**. To model interactions, developers must either:

1. Pass external "Data/Matter" arguments to every method (e.g., `solve_j2(&self, data: &[Coord])`).
2. Wrap the field in a container struct (Wrapper Pattern), leading to type proliferation and loss of trait ergonomics.

### 2.2 The "Inverse Problem" Gap

In "Inverse Physics" (Metrology), the field is derived *from* observations. The observations (Source) are intrinsic to
the field's existence. Separating them creates an architectural disconnect where the field "doesn't know where it came
from."

## 3. Proposed Solution: Gauge-Source Unification

We propose elevating the **Source** to a first-class generic member of the field struct.

### 3.1 Structural Definition

The struct definition will be updated to include a generic `S`:

```rust
#[derive(Debug, Clone)]
pub struct LatticeGaugeField<G: GaugeGroup, const D: usize, M, R, S = ()> {
    /// The underlying lattice structure (Topology).
    lattice: Arc<Lattice<D>>,

    /// Link variables U_μ(x) (The Mediator).
    links: HashMap<LatticeCell<D>, LinkVariable<G, M, R>>,

    /// Coupling parameter β (Interaction Strength).
    beta: R,

    /// The Source that generates the field.
    /// Defaults to () for Vacuum.
    source: S,
}
```

### 3.2 Design Principles

1. **Backward Compatibility**: By defaulting `S = ()`, existing code using `LatticeGaugeField<G, D, M, R>` remains
   valid (it implicitly becomes `... R, ()>`).
2. **Structural Unity**: The Field ($U$) and Source ($\psi$) typically appear together in the
   Lagrangian: $\mathcal{L} = \mathcal{L}_{gauge}(U) + \mathcal{L}_{source}(\psi, U)$. This struct reflects that
   physical unity.
3. **Encapsulation**: Operations dependent on the source (e.g., Energy-Momentum Tensor calculation) can now be methods
   on `self` rather than taking external arguments.

---

## 4. Technical Specification

### 4.1 Accessor API

Since `source` is private, we require protected access similar to `links`.

```rust
impl<G, const D: usize, M, R, S> LatticeGaugeField<G, D, M, R, S> {
    /// Returns a reference to the field source.
    pub fn source(&self) -> &S {
        &self.source
    }

    /// Returns a mutable reference to the field source.
    /// (Useful for dynamic matter evolution like Monte Carlo updates of Fermions)
    pub fn source_mut(&mut self) -> &mut S {
        &mut self.source
    }

    /// Replaces the source and returns the old one.
    pub fn replace_source(&mut self, new_source: S) -> S {
        std::mem::replace(&mut self.source, new_source)
    }
}
```

### 4.2 Constructor API

Constructors must be updated to handle `S`.

**Default (Vacuum) Constructors**:
Existing constructors (`identity`, `random`) will initialize `source: S::default()` (requiring `S: Default`).

**Explicit Constructors**:
New builders to attach sources during creation.

```rust
impl<G, const D: usize, M, R, S> LatticeGaugeField<G, D, M, R, S> {
    /// Attaches a source to an existing field, transforming the type.
    /// Consumes the vacuum field and returns a sourced field.
    pub fn with_source<NewS>(self, source: NewS) -> LatticeGaugeField<G, D, M, R, NewS> {
        LatticeGaugeField {
            lattice: self.lattice,
            links: self.links,
            beta: self.beta,
            source,
        }
    }
}
```

### 4.3 Interaction Methods

The unification allows implementing interaction terms directly on the field.

```rust
// Theoretical Example: Interaction Energy
impl < ..., S> LatticeGaugeField<..., S>
where S: MatterField {
pub fn interaction_action(&self) -> R {
    // S_int = <psi_bar | gamma_mu A_mu | psi>
    // Can access self.links AND self.source
    let matter = self.source();
    let links = &self.links;
    // ... calculation ...
}
}
```

---

## 5. Use Cases

### 5.1 Case A: The Vacuum (Standard LGT)

* **Type**: `LatticeGaugeField<SU3, 4, Matrix3, f64>` (Implicit `S=()`)
* **Physics**: Pure Glue QCD (Yang-Mills).
* **Behavior**: `source` is Unit. Zero overhead. Behaves exactly as current implementation.

### 5.2 Case B: Dynamic Matter (Lattice QCD)

* **Type**: `LatticeGaugeField<SU3, 4, Matrix3, f64, FermionMatrix>`
* **Physics**: Full QCD with Quarks.
* **Behavior**: `source` holds the Pseudo-fermion field $\phi$. HMC (Hybrid Monte Carlo) updates evolve both `links` (
  Gauge) and `source` (Matter) in tandemsteps.

### 5.3 Case C: Inverse Metrology (Chrono-Gauge)

* **Type**: `LatticeGaugeField<SU2_U1, 4, Complex, f64, Vec<Observation>>`
* **Physics**: Gravity derived from Clock Data.
* **Behavior**: `source` holds the `SpaceTimeCoordinates`.
    * `solve_j2()` becomes `self.solve_j2()`, using internal data.
    * `source()` (Einstein Inversion) uses `self.source()` to get data and `self.links()` to get frame-dragging
      corrections.

### 5.4 Case D: Kinetic Theory (Kinetic-Gauge)

* **Type**: `LatticeGaugeField<SE3, 4, Matrix4, f64, PhaseSpace>`
* **Physics**: Accelerated frames.
* **Behavior**: `source` holds `(Position, Momentum)` of test particles. The Gauge Field (Connection) describes the
  inertial forces acting on them.

---

## 6. Migration & Impact

1. **Core Library**: Modify `deep_causality_topology`.
    * Change struct definition.
    * Update all `impl` blocks to carry generic `S`.
    * Update constructors.
2. **Downstream**:
    * **No Break** for type aliases (e.g., `type MyField = LatticeGaugeField<...>`). Generics default to `()` so
      `MyField` remains valid.
    * **Break** for struct literals: `LatticeGaugeField { lattice, ... }` will need `source: ()` added.
3. **Chrono-Physics**:
    * Update `ChronoGauge` alias to include `Vec<SpaceTimeCoordinate>`.
    * Refactor operations to use internal source.

## 7. Conclusion

Adding `Source` as a first-class citizen aligns the data structure with the physical Lagrangians it simulates. It
bridges the gap between purely mathematical simulations and data-driven physics, enabling the "Hybrid" architecture
required for high-precision metrology without resort to ad-hoc wrappers.
