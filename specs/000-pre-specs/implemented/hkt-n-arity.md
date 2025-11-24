# Higher-Kinded Types (HKT) N-Arity Specification

This document specifies the design for Higher-Kinded Types (HKT) of Arity 2 through 5, along with the functional traits that leverage them. These traits extend the capabilities of `deep_causality_haft` to support complex multi-variable systems found in Quantum Computing, Magnetohydrodynamics (MHD), and Causal Topology.

## 1. Core HKT Definitions (Unbound)

The existing `HKT2`, `HKT3`, etc., in `deep_causality_haft` are "Fixed" or "Partially Applied" HKTs (e.g., `HKT2<F>` fixes one type and leaves one open).

To support traits like `Bifunctor` (where *both* types change), we introduce **Unbound** HKT traits. These traits abstract over type constructors with $N$ generic parameters, all of which are free to vary.

### 1.1 Arity 2: `HKT2Unbound`

Abstracts over types like `Result<A, B>`, `Either<L, R>`, or `Pair<A, B>`.

```rust
/// Witness trait for a type constructor with 2 generic parameters: F<A, B>
pub trait HKT2Unbound {
    /// The Generic Associated Type representing F<A, B>
    type Type<A, B>;
}

// Example Witness
pub struct ResultWitness;
impl HKT2Unbound for ResultWitness {
    type Type<A, B> = Result<A, B>;
}
```

### 1.2 Arity 3: `HKT3Unbound`

Abstracts over types like `Triple<A, B, C>` or Parametric States `State<S_in, S_out, A>`.

```rust
/// Witness trait for a type constructor with 3 generic parameters: F<A, B, C>
pub trait HKT3Unbound {
    type Type<A, B, C>;
}
```

### 1.3 Arity 4: `HKT4Unbound`

Abstracts over types like `Tetrad<A, B, C, D>` or Interaction Tensors.

```rust
/// Witness trait for a type constructor with 4 generic parameters: F<A, B, C, D>
pub trait HKT4Unbound {
    type Type<A, B, C, D>;
}
```

### 1.4 Arity 5: `HKT5Unbound`

Abstracts over types like `Pentad<S, B, C, A, E>` for Cybernetic Loops.

```rust
/// Witness trait for a type constructor with 5 generic parameters: F<A, B, C, D, E>
pub trait HKT5Unbound {
    type Type<A, B, C, D, E>;
}
```

---

## 2. Phase 1: Arity-2 Traits (The "Relations" Layer)

These traits handle systems with two moving parts.

### 2.1 `Bifunctor` (Parallel Evolution)

**Concept:** Maps two independent types simultaneously. $F<A, B> \to F<C, D>$.
**Use Case:** Evolving a `System<Topology, Algebra>` where both the mesh and the values change.

```rust
pub trait Bifunctor<F: HKT2Unbound> {
    /// Map both types simultaneously.
    fn bimap<A, B, C, D, F1, F2>(
        fab: F::Type<A, B>,
        f1: F1,
        f2: F2
    ) -> F::Type<C, D>
    where
        F1: FnMut(A) -> C,
        F2: FnMut(B) -> D;

    /// Map only the first type.
    fn first<A, B, C, F1>(
        fab: F::Type<A, B>,
        f1: F1
    ) -> F::Type<C, B>
    where
        F1: FnMut(A) -> C,
    {
        Self::bimap(fab, f1, |b| b)
    }

    /// Map only the second type.
    fn second<A, B, D, F2>(
        fab: F::Type<A, B>,
        f2: F2
    ) -> F::Type<A, D>
    where
        F2: FnMut(B) -> D,
    {
        Self::bimap(fab, |a| a, f2)
    }
}
```

### 2.2 `Profunctor` (The Compiler Interface)

**Concept:** $P<A, B>$. Contravariant on Input ($A$), Covariant on Output ($B$).
**Use Case:** Adapting a core kernel to different hardware interfaces (HAL).

```rust
pub trait Profunctor<P: HKT2Unbound> {
    /// Contravariant map on A (Input), Covariant map on B (Output).
    /// "Pre-process the input, Post-process the output."
    fn dimap<A, B, C, D, F1, F2>(
        pab: P::Type<A, B>,
        f_pre: F1,   // C -> A (Pre-processor)
        f_post: F2   // B -> D (Post-processor)
    ) -> P::Type<C, D>
    where
        F1: FnMut(C) -> A,
        F2: FnMut(B) -> D;

    /// Map only the input (Contravariant).
    fn lmap<A, B, C, F1>(
        pab: P::Type<A, B>,
        f_pre: F1
    ) -> P::Type<C, B>
    where
        F1: FnMut(C) -> A,
    {
        Self::dimap(pab, f_pre, |b| b)
    }

    /// Map only the output (Covariant).
    fn rmap<A, B, D, F2>(
        pab: P::Type<A, B>,
        f_post: F2
    ) -> P::Type<A, D>
    where
        F2: FnMut(B) -> D,
    {
        Self::dimap(pab, |a| a, f_post)
    }
}
```

### 2.3 `Adjunction` (The Stokes' Theorem Trait)

**Concept:** Two functors $L$ and $R$ are adjoint if $L(A) \to B \cong A \to R(B)$.
**Use Case:** Exact conservation laws. Linking Boundary Operators ($\partial$) and Exterior Derivatives ($d$).

```rust
use crate::HKT;

pub trait Adjunction<L, R>
where
    L: HKT,
    R: HKT,
{
    /// The Left Adjunct: (L(A) -> B) -> (A -> R(B))
    /// "Turning a boundary integral into a volume integral."
    fn left_adjunct<A, B, F>(
        a: A,
        f: F
    ) -> R::Type<B>
    where
        F: Fn(L::Type<A>) -> B;

    /// The Right Adjunct: (A -> R(B)) -> (L(A) -> B)
    fn right_adjunct<A, B, F>(
        la: L::Type<A>,
        f: F
    ) -> B
    where
        F: Fn(A) -> R::Type<B>;
        
    /// The Unit: A -> R(L(A))
    fn unit<A>(a: A) -> R::Type<L::Type<A>>;
    
    /// The Counit: L(R(B)) -> B
    fn counit<B>(lrb: L::Type<R::Type<B>>) -> B;
}
```

---

## 3. Phase 2: Arity-3 Traits (The "Dynamics" Layer)

### 3.1 `ParametricMonad` (The "Indexed" Monad)

**Concept:** $M<S_{in}, S_{out}, A>$. A computation that changes the *Type* of the state.
**Use Case:** Phase transitions (Fluid $\to$ Crystal) or Topology Rewrites.

```rust
pub trait ParametricMonad<M: HKT3Unbound> {
    /// Inject a value into a computation that doesn't change state type (S -> S).
    fn pure<S, A>(value: A) -> M::Type<S, S, A>;

    /// Indexed Bind: Chain computations where the state type evolves.
    /// Step 1: S1 -> S2
    /// Step 2: S2 -> S3
    /// Result: S1 -> S3
    fn ibind<S1, S2, S3, A, B, F>(
        m: M::Type<S1, S2, A>,
        f: F
    ) -> M::Type<S1, S3, B>
    where
        F: FnMut(A) -> M::Type<S2, S3, B>;
}
```

### 3.2 `Promonad` (The Interaction Trait)

**Concept:** Maps $(A, B) \to C$ within a context. Models "Synergy" or "Fusion".
**Use Case:** Tensor Contraction, Quantum Entanglement, Force Calculation ($J \times B = F$).

```rust
pub trait Promonad<P: HKT3Unbound> {
    /// Merge two contexts into a third.
    /// This is a generalization of `zip_with` or `liftA2` but for distinct types.
    fn merge<A, B, C, F>(
        pa: P::Type<A, A, A>, // Simplified for illustration, usually P<A, B, C> context
        pb: P::Type<B, B, B>,
        f: F
    ) -> P::Type<C, C, C>
    where
        F: FnMut(A, B) -> C;
        
    // Note: The signature of Promonad can vary. 
    // A strict "Pre-Arrow" definition might be better suited:
    // P<A, B, C> where A and B are inputs and C is output.
    
    /// Fusion: Combine Input A and Input B to produce Output C
    fn fuse<A, B, C>(
        input_a: A,
        input_b: B
    ) -> P::Type<A, B, C>;
}
```

---

## 4. Phase 3: High-Arity Traits (The "Universe" Layer)

### 4.1 Arity 4: `RiemannMap` (Curvature & Scattering)

**Concept:** $R(A, B, C) \to D$. Consumes 3 resources to produce 1, or maps 2 to 2.
**Use Case:** Riemann Curvature Tensor ($R(u, v)w$), QFT Scattering Matrices.

```rust
pub trait RiemannMap<P: HKT4Unbound> {
    /// The Curvature Operator: R(u, v)w
    /// Consumes two directions (u, v) and a vector (w) to measure curvature (result).
    fn curvature<A, B, C, D>(
        tensor: P::Type<A, B, C, D>,
        u: A,
        v: B,
        w: C
    ) -> D;

    /// The Scattering Matrix: In(A, B) -> Out(C, D)
    /// Used for Particle Physics (Standard Model interactions).
    fn scatter<A, B, C, D>(
        interaction: P::Type<A, B, C, D>,
        in_1: A,
        in_2: B
    ) -> (C, D);
}
```

### 4.2 Arity 5: `CyberneticLoop` (The Agent)

**Concept:** A complete feedback loop involving 5 distinct types.
**Use Case:** Autonomous Agents, Quantum Error Correction.

1.  **Sensor ($S$)**
2.  **Belief ($B$)**
3.  **Context ($C$)**
4.  **Action ($A$)**
5.  **Entropy ($E$)**

```rust
pub trait CyberneticLoop<P: HKT5Unbound> {
    /// The "OODA Loop" (Observe, Orient, Decide, Act) as a Type Signature.
    /// 
    /// This function proves that an Agent can consume Sensor data (S),
    /// update its Belief (B) based on Context (C), and produce an Action (A),
    /// while accounting for Entropy (E).
    fn control_step<S, B, C, A, E, F_Observe, F_Decide>(
        agent: P::Type<S, B, C, A, E>,
        sensor_input: S,
        observe_fn: F_Observe,
        decide_fn: F_Decide
    ) -> Result<A, E>
    where
        F_Observe: Fn(S, C) -> B, // Raw -> Belief
        F_Decide:  Fn(B, C) -> A; // Belief -> Action
}
```

## 5. Applied Examples & Scientific Advancement

The following examples demonstrate how these HKT traits transform scientific computing from "calculating numbers" to "modeling physics."

### 5.1 Example 1: The Geometry Engine (Einstein Field Equations)

**The Problem:**
In traditional physics engines (and the current `deep_causality_tensor` example), the Einstein Field Equations are solved by manually manipulating raw 4D arrays.
*   **Risk:** You can accidentally multiply a "Temperature Tensor" by a "Metric Tensor" because they are both just `f64` arrays.
*   **Opacity:** The code calculates `R_uv` but doesn't express *why* it exists.

**The Solution (`RiemannMap`):**
We replace the hardcoded data arrays with a **Geometry Engine** that enforces the laws of General Relativity at the type level.

```rust
// OLD: Hardcoded Data Arrays
// The compiler sees this as just a 4x4 grid of floats.
let r_uv = CausalTensor::new(ricci_data, shape).unwrap();
let g_uv = CausalTensor::new(metric_data, shape).unwrap();
let g_tensor = r_uv + (-0.5 * r * g_uv); // Manual arithmetic

// NEW: The Geometry Engine (HKT4)
// The compiler sees this as a Curvature Operator on a Manifold.
struct SpacetimeGeometry;

impl RiemannMap<CausalTensorWitness> for SpacetimeGeometry {
    /// Calculates R(u,v)w -> D
    /// This function GUARANTEES that the output is a valid tangent vector
    /// derived from the connection (Christoffel symbols).
    fn curvature<A, B, C, D>(
        metric: CausalTensor<f64>, // The Geometry
        u: A,                      // Direction 1 (Time)
        v: B,                      // Direction 2 (Radial)
        w: C                       // Vector to transport
    ) -> D {
        // ... Internal implementation using Christoffel symbols ...
    }
}

// Usage: Measure gravity's effect on a particle
// We don't just "add tensors"; we ask the geometry how it curves.
let gravity_effect = SpacetimeGeometry::curvature(
    g_uv,
    time_axis,
    radial_axis,
    particle_vector
);
```

### 5.2 Example 2: The Coupled Universe (GRMHD)

**The Problem:**
Coupling General Relativity (GR) with Magnetohydrodynamics (MHD) usually involves "Ad-Hoc Coupling."
*   **Lossy:** The GR solver runs, outputs a single number (e.g., "Time Dilation Factor"), and the MHD solver reads it.
*   **Disconnect:** The Plasma doesn't "feel" the shape of the Black Hole, only a scalar approximation.

**The Solution (`Bifunctor` & `Promonad`):**
We treat the Universe as a **Pair** of systems that evolve together, and their interaction as a **Tensor Product**.

```rust
// 1. The Coupled State (Bifunctor)
// The Universe is NOT two separate variables; it is a single entangled state.
type Universe = Pair<CausalTensor<f64>, CausalMultiVector<f64>>;

// Evolve both simultaneously without unzipping.
// This ensures the Plasma and the Metric are always in sync (same time step).
let next_state = UniverseWitness::bimap(
    current_state,
    |g| evolve_gravity(g), // GR Solver (Metric Evolution)
    |p| evolve_plasma(p)   // MHD Solver (Fluid Dynamics)
);

// 2. The Interaction (Promonad)
// The Lorentz Force (F = J x B) is calculated via a Merge.
// The Metric (g) is injected into the context, allowing the Cross Product
// to be defined correctly in Curved Space.
let force = CausalTensorWitness::merge(
    current_j,        // Plasma Current
    magnetic_field_b, // Magnetic Field
    |j, b| j.cross_product(b, next_state.first()) // Cross Product in Curved Space
);
```

---

## 6. Advancing Science: Why This Matters

This HKT-based approach represents a paradigm shift in scientific computing.

### 6.1 Type-Safe Geometry
In C++ or Fortran, a `Vector3` is just 3 numbers. In `deep_causality`, a `Vector3` in a `RiemannMap` is a specific geometric object (Tangent Vector).
*   **Advancement:** We eliminate entire classes of "Category Errors" in physics simulations. You cannot plug a "Voltage" into a "Curvature" slot. The compiler enforces physical dimensional consistency.

### 6.2 Compositional Physics
Current physics engines are monolithic solvers. With HKTs, we build **Standard Models** from reusable components.
*   **Advancement:**
    *   **Geometry Component:** `RiemannMap` (Swappable: Black Hole, Warp Drive, Wormhole).
    *   **Interaction Component:** `Promonad` (Swappable: Electromagnetism, Weak Force, Strong Force).
    *   **Agent Component:** `CyberneticLoop` (Swappable: PID Controller, AI Agent, Quantum Error Correction).
    *   You can compose a "Quantum AI controlling a Fusion Reactor near a Black Hole" simply by stacking these traits.

### 6.3 The "Ridiculous" Factor
Implementing this in Rust with Zero-Cost Abstractions allows us to encode the *structure* of the universe into the type system. This allows for **Formal Verification** of physical laws, something impossible with standard numerical arrays.
