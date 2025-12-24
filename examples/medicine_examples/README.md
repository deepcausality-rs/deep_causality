# Medicine Examples

Examples demonstrating DeepCausality applications in biomedical and life sciences domains.

## Quick Start

Run any example from the repository root:

```bash
cargo run -p medicine_examples --example <example_name>
```

---

## Examples Overview

| Example                                                               | Domain | Description |
|-----------------------------------------------------------------------|--------|-------------|
| [aneurysm_risk](aneurysm_risk/README.md)                      | Cardiovascular | Aneurysm rupture risk via fluid fatigue accumulation |
| [diving_decompression](diving_decompression/README.md)        | Hyperbaric | Bühlmann ZH-L16C decompression with CNS O2 toxicity |
| [epilepsy](epilepsy/README.md)                               | Neurology | Virtual surgery planning using brain network digital twins |
| [protein_folding](protein_folding/README.md)                 | Biophysics | Non-Markovian protein dynamics with memory kernels |
| [tissue_classification](tissue_classification/README.md) | Medical Imaging | Topological tissue analysis for tumor detection |
| [tumor_treatment](tumor_treatment/README.md)                 | Oncology | Optimizing TTFields therapy with Geometric Algebra |

---

## Key Concepts

### Non-Markovian Dynamics

Many biological systems exhibit **memory effects** - current behavior depends on history, not just the present state. The Generalized Master Equation (GME) captures this:

```
P(t+Δt) = T · P(t) + Σ K(τ) · P(t-τ)
```

### Topological Data Analysis (TDA)

Structural features (holes, voids) in tissue can indicate pathology. The Euler Characteristic provides a robust topological classifier.

### Applications

- **Drug Discovery**: Understanding protein misfolding (Alzheimer's, Parkinson's)
- **Tumor Detection**: Identifying necrotic cores via topology
- **Bioengineering**: Designing proteins with specific functions
- **Medical Imaging**: Robust tissue classification

---

## Crates Used

| Crate | Purpose |
|-------|---------|
| `deep_causality_physics` | Generalized Master Equation |
| `deep_causality_tensor` | Transition matrices, point clouds |
| `deep_causality_topology` | Simplicial complexes, TDA |
| `deep_causality_core` | Monadic effects |

---

## See Also

- [physics_examples](../physics_examples/README.md) - Pure physics simulations
- [case_study_icu_sepsis](../case_study_icu_sepsis/) - Clinical sepsis prediction
