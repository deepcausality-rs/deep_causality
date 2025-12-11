# Multi-Physics Pipeline: Grand Unification

This example demonstrates a complete multi-physics simulation chain: Quantum Field Theory → Hadronization → Hydrodynamics → Detection.

## How to Run

```bash
cargo run -p physics_examples --example multi_physics_pipeline
```

---

## Engineering Value

Multi-physics pipelines are essential for:
- **Particle Physics**: Simulating collider experiments (LHC)
- **Astrophysics**: Supernova, neutron star simulations
- **Plasma Physics**: Fusion reactor modeling

This example shows **zero-copy data transformation** across 3 distinct physics domains.

---

## Physics Background

### The Pipeline

Models what happens in a particle collision:

1. **Quantum Field (Klein-Gordon)**: High-energy scalar field evolution
2. **Hadronization**: Quarks/gluons → observable hadrons
3. **Hydrodynamics**: Thermal expansion of particle cloud
4. **Detection**: Quantum measurement probability

### Higgs-Like Decay

```text
φ (scalar field) → Energy density → Jets → Thermal plasma → Detection
```

---

## Causal Chain

```text
[Step 1] Klein-Gordon Evolution
         φ_initial → klein_gordon() → φ_evolved
                        ↓
[Step 2] Energy Density Calculation
         φ² × 100 → energy densities
                        ↓
[Step 3] Hadronization
         hadronization() → particle jets
                        ↓
[Step 4] Thermalization
         jets → temperature field
                        ↓
[Step 5] Heat Diffusion
         heat_diffusion() → thermal equilibrium
                        ↓
[Step 6] Quantum Detection
         born_probability() → measurement probability
```

---

## Output Interpretation

```
-> Generated 8 Particle Jets/Hadrons
-> Final Quark-Gluon Plasma Temp: 247.50 K
-> Detection Probabilities: 0.0613
```

The pipeline transforms initial quantum field to final detection probability.

---

## Adapting This Example

1. **Different initial conditions**: Vary mass, initial field configuration
2. **3D simulation**: Extend to full 3D spatial grid
3. **More physics**: Add electroweak, QCD corrections
4. **Detector geometry**: Model realistic detector acceptance

---

## Key APIs Used

- `klein_gordon()` - Scalar field evolution
- `hadronization()` - Particle production
- `heat_diffusion()` - Thermal dynamics
- `born_probability()` - Quantum measurement
- `bind_or_error()` - Error-handling composition
