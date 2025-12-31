# Multi-Physics Pipeline: QFT → QCD → Thermal → Detection

This example demonstrates **modular composition** via the **Causal Monad** (`CausalEffectPropagationProcess`)
for a complete high-energy physics simulation chain.

## How to Run

```bash
cargo run -p physics_examples --example multi_physics_pipeline
```

## Sample Output

```
═══════════════════════════════════════════════════════════════
  Multi-Physics Pipeline: QFT → QCD → Thermal → Detection
  (Modular Stages Composed via Causal Monad)
═══════════════════════════════════════════════════════════════

Stage 1: Klein-Gordon Scalar Field
  Field energy: E_cms = 500.00 GeV

Stage 2: QCD String Creation
  q:  (E=250.0, pz=+250.0) GeV
  q̄:  (E=250.0, pz=-250.0) GeV

Stage 3: Lund String Fragmentation
  Produced 101 hadrons (61 physical)
  Sample hadrons:
    [ 1] π⁺ (PDG 211): E = 167.91 GeV
    [ 2] π⁻ (PDG -211): E = 147.19 GeV
    ...

Stage 4: Thermalization
  Initial temp: 500.0 MeV
  Equilibrium:  455.0 MeV

Stage 5: Quantum Detection
  Critical temp: T_c = 170 MeV
  |ψ⟩ = 0.853|QGP⟩ + 0.522|hadron⟩
  P(QGP detection) = 0.7280

[SUCCESS] Modular Pipeline Completed.
```

---

## Key Pattern: Causal Monad Composition

The power of this example is the **decoupled, modular pipeline**:

```rust
let result = klein_gordon( & phi_manifold, mass)
.bind_or_error(stage_field_to_partons, "...")
.bind_or_error(stage_lund_fragmentation, "...")
.bind_or_error(stage_thermalization, "...")
.bind_or_error(stage_quantum_detection, "...");
```

Each stage is a **standalone function** that can be:

- ✅ **Tested independently**
- ✅ **Replaced without affecting other stages**
- ✅ **Reused in different pipelines**
- ✅ **Extended with new physics**

---

## Physics Pipeline

```
Klein-Gordon Field  →  Virtual q-q̄ Creation  →  Lund Fragmentation  →  Thermalization  →  Detection
     φ(x,t)         →    FourMomentum pairs   →    π, K, ρ, ω, η    →     T(x,t)        →   P(QGP)
```

| Stage | Physics                                 | API                                  |
|-------|-----------------------------------------|--------------------------------------|
| 1     | Scalar field evolution: (□ + m²)φ = 0   | `klein_gordon()`                     |
| 2     | Virtual q-q̄ creation from field energy | Manual conversion                    |
| 3     | QCD string fragmentation (PYTHIA-like)  | `lund_string_fragmentation_kernel()` |
| 4     | Heat diffusion: ∂T/∂t = κ∇²T            | `heat_diffusion()`                   |
| 5     | Born probability: P =                   | ⟨basis\|ψ⟩                           |² | `born_probability()` |

---

## ⚠️ Simplifications in This Example

This is a **pedagogical demonstration**.

| Aspect                  | This Example           | Production Reality             |
|-------------------------|------------------------|--------------------------------|
| **Collision system**    | e⁺e⁻ → q-q̄ (1 string) | Pb-Pb → QGP (1000s of partons) |
| **Hadron multiplicity** | ~60 hadrons            | ~10,000+ in heavy-ion          |
| **QGP formation**       | Instant                | Thermalization ~1 fm/c         |
| **Temperature**         | Scaled from energy     | From particle spectra fits     |
| **Detection**           | Simple P = T/(T+T_c)   | Jet quenching, flow, spectra   |

### Key Simplifications

1. **No actual QGP**: Single q-q̄ string → hadrons directly, no plasma phase
2. **1D manifold**: Real simulations use 3+1D spacetime grids
3. **Simplified Lund**: Full PYTHIA has parton showers, color reconnection
4. **Thermalization**: Real uses relativistic hydrodynamics (MUSIC, vHLLE)
5. **Detection**: Real uses detector geometry, efficiency, backgrounds

---

## 🚀 Path to Production Code

To evolve this example into realistic simulation:

### Stage 1: Replace Initial Conditions

```diff
- Klein-Gordon 1D field
+ Glauber model for heavy-ion geometry
+ EKRT/IP-Glasma initial state
+ CGC saturation physics
```

### Stage 2: Full Parton Shower

```diff
- Single q-q̄ string
+ PYTHIA 8 parton shower
+ Final-state radiation (FSR)
+ Initial-state radiation (ISR)
+ Color reconnection
```

### Stage 3: Realistic Fragmentation

```diff
- Simplified Lund kernel
+ Full PYTHIA string fragmentation
+ Heavy quark fragmentation (Peterson, etc.)
+ Baryon production (diquark model)
```

### Stage 4: Relativistic Hydrodynamics

```diff
- 1D heat diffusion
+ 3+1D relativistic viscous hydro
+ Equation of state from lattice QCD
+ Cooper-Frye freeze-out
```

### Stage 5: Full Detector Simulation

```diff
- Simple Born probability
+ GEANT4 detector simulation
+ Track reconstruction, PID
+ Jet finding (anti-kT)
+ Observable calculations (v₂, R_AA, etc.)
```

### Architecture for Production

```rust
// Production version with proper physics modules
let result = initial_state::glauber( & nucleus_a, & nucleus_b)
.bind_or_error(parton_shower::pythia8, "Shower failed")
.bind_or_error(fragmentation::lund_full, "Frag failed")
.bind_or_error(hydro::music_3d, "Hydro failed")
.bind_or_error(detector::geant4_alice, "Detector failed")
.bind_or_error(analysis::jet_quenching, "Analysis failed");
```

The **Causal Monad pattern remains the same** — only the stage implementations change.

---

## Key APIs Used

| API                                  | Purpose                             |
|--------------------------------------|-------------------------------------|
| `CausalEffectPropagationProcess`     | Causal Monad for composition        |
| `bind_or_error()`                    | Monadic bind with error propagation |
| `klein_gordon()`                     | Scalar field dynamics               |
| `lund_string_fragmentation_kernel()` | QCD hadronization                   |
| `heat_diffusion()`                   | Thermal physics                     |
| `born_probability()`                 | Quantum measurement                 |

---

## Engineering Value

This pattern is applicable to any multi-stage simulation:

- **Particle Physics**: LHC event generation
- **Astrophysics**: Supernova, neutron star mergers
- **Plasma Physics**: Tokamak/stellarator fusion
- **Climate**: Atmosphere-ocean-ice coupling
- **Finance**: Multi-factor risk modeling

The key insight: **Decouple physics stages for maintainability, compose with monads for correctness.**
