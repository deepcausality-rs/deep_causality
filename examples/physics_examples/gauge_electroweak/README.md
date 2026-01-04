# Electroweak Unification (SU(2)×U(1)) Example

This example demonstrates the **Electroweak Theory** in DeepCausality, unifying electromagnetic and weak forces.

## Overview

The Electroweak model unifies QED (photon) and Weak (W/Z bosons) forces through spontaneous symmetry breaking.
This pipeline simulates:

1. **Unification**: Calculates relationships between coupling constants (α, g, g').
2. **Symmetry Breaking**: Computes mass generation for W and Z bosons via the Higgs mechanism.
3. **Gauge Mixing**: Verifies the W/Z mass ratio and Weinberg angle relationships.
4. **Z Resonance**: Calculates the Breit-Wigner cross-section for Z boson production.

## Key Concepts

- **Weinberg Angle θ_W**: The mixing angle between B and W³ fields.
- **Higgs Mechanism**: Generation of mass terms via vacuum expectation value (v).
- **Unification Condition**: e = g sin θ_W = g' cos θ_W.

## Running

```bash
cargo run --example gauge_electroweak -p physics_examples
```

## Expected Output

```
Stage 1: Unification
  Weinberg Angle:     sin²θ_W = 0.2312
  EM Coupling (e):    0.3028

Stage 2: Spontaneous Symmetry Breaking
  Higgs VEV (v):      246.22 GeV
  Generated M_W:      80.37 GeV
  Generated M_Z:      91.19 GeV

Stage 3: Gauge Boson Mixing
  ρ parameter:        1.0107 (Tree level SM = 1.0)
```
