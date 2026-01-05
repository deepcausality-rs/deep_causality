# Weak Force (SU(2)) Example

This example demonstrates the **Weak Nuclear Force** using the specialized DeepCausality physics module.

## Overview

The Weak Force is responsible for radioactive decay (like beta decay) and neutrino interactions.
This pipeline simulates:

1. **Particle Initialization**: Creation of left-handed lepton doublets (SU(2) states).
2. **Charged Current (CC)**: W boson exchange (e.g., muon decay).
   - Demonstrates the W propagator at low energy.
3. **Neutral Current (NC)**: Z boson exchange (e.g., neutrino scattering).
   - Calculates effective couplings g_V and g_A.
4. **Decay Analysis**:
   - Computes muon lifetime from Fermi theory.
   - Estimates W and Z boson decay widths.

## Key Concepts

- **Weak Isospin**: Left-handed fermions have isospin I=1/2.
- **W/Z Propagators**: 1/(q² - M²) dependence.
- **Fermi Constants**: G_F determines the strength of weak interactions at low energy.

## Running

```bash
cargo run --example gauge_weak -p physics_examples
```

## Expected Output

```
Stage 1: Particle Initialization
  Muon (L):      I = 0.5, I3 = -0.5, Q = -1
  Neutrino (L):  I = 0.5, I3 = 0.5,  Q = 0

Stage 2: Charged Current
  W Propagator:  -1.5473e-4 GeV⁻²

Stage 3: Neutral Current
  Z Propagator:  5.9xxxe-5 GeV⁻²

Stage 4: Decay Properties
  Muon lifetime: 2.19xx e-06 s
  W Boson width: 2.xxx GeV
```
