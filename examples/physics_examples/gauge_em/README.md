# Gauge EM: Relativistic Electrodynamics Pipeline

This example demonstrates **gauge-theoretic electromagnetism** analysis
using the **Causal Monad** (`PropagatingEffect`) for type-safe, modular composition of
physics stages.

## Running

```bash
RUSTFLAGS='-C target-cpu=native' cargo run --example gauge_em -p physics_examples --release
```

## Overview

The pipeline processes an electromagnetic plane wave through a sequence of independent
analysis stages, each composed via `bind_or_error`:

```
Stage 1: Create Plane Wave (with U(1) gauge field structure)
       ↓
Stage 2: Compute Lorentz Invariants (F_μν F^μν, F_μν F̃^μν)
       ↓
Stage 3: Energy Analysis (u, L)
       ↓
Stage 4: Radiation Analysis (Poynting vector, intensity)
       ↓
Stage 5: Field Classification (radiation/null)
       ↓
    Summary
```

## Key Physics Concepts

### U(1) Gauge Field Structure

The electromagnetic field is represented using relativistic gauge theory:

| Property | Value |
|----------|-------|
| Gauge Group | U(1) |
| Lie Algebra | u(1), dim = 1 |
| Abelian | Yes (F = dA) |
| Connection | A_μ (4-potential) |
| Curvature | F_μν = ∂_μA_ν - ∂_νA_μ |
| Metric | West Coast (+---) |

### Lorentz Invariants

- **Field invariant**: `F_μν F^μν = 2(B² - E²)` — same in all reference frames
- **Dual invariant**: `F_μν F̃^μν = -4 E·B` — measures CP violation

### Energy Quantities

- **Energy density**: `u = (E² + B²)/2` — the T^{00} component of stress-energy
- **Lagrangian density**: `L = (E² - B²)/2` — the EM Lagrangian

### Radiation Properties

- **Poynting vector**: `S = E × B` — energy flux (power per unit area)
- **Intensity**: `|S|` — magnitude of energy flux

### Field Classification

| Property           | Condition     | Example    |
|--------------------|---------------|------------|
| Radiation          | E ⟂ B         | Plane wave |
| Null field         | \|E\| = \|B\| | Light wave |
| Electric-dominated | \|E\| > \|B\| | Capacitor  |
| Magnetic-dominated | \|B\| > \|E\| | Solenoid   |

## Running the Example

```bash
cargo run --example gauge_em -p physics_examples
```

## Design Pattern: Causal Monad

This example showcases the **Causal Monad** pattern using `bind_or_error`:

```rust
let result = create_plane_wave()
.bind_or_error(stage_compute_invariants, "Invariant computation failed")
.bind_or_error(stage_energy_analysis, "Energy analysis failed")
.bind_or_error(stage_poynting_radiation, "Radiation analysis failed")
.bind_or_error(stage_field_classification, "Field classification failed");
```

### Benefits

1. **Type-Safe Error Propagation**: Errors automatically propagate through the chain
2. **Modular Stages**: Each stage is an independent function that can be:
    - Tested in isolation
    - Replaced without affecting other stages
    - Reused in different pipelines
3. **Clean Composition**: No nested error handling or match statements
4. **Explicit Failure Points**: Each `bind_or_error` has a descriptive error message

## Code Structure

```
gauge_em/
├── main.rs    # Pipeline composition and stage implementations
└── README.md  # This file
```

## Related Examples

- [`multi_physics_pipeline`](../multi_physics_pipeline/) — QFT → QCD → Thermal → Detection
- [`maxwell`](../maxwell/) — Classical EM field propagation
- [`gravitational_wave`](../gravitational_wave/) — GR wave detection

## References

- Jackson, *Classical Electrodynamics*, Chapter 6 (EM Energy and Momentum)
- Peskin & Schroeder, *QFT*, Chapter 2 (Gauge Field Formalism)
- [deep_causality_physics::GaugeEM](../../deep_causality_physics/src/theories/gauge_em/)
