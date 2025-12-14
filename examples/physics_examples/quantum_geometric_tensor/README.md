# Quantum Geometric Tensor Example

Demonstrates the **Quantum Geometric Tensor (QGT)** and its connection to observable transport properties in condensed matter systems like **Twisted Bilayer Graphene (TBG)**.

## How to Run

```bash
cargo run -p physics_examples --example quantum_geometric_tensor
```

---

## Physics Overview

The **Quantum Geometric Tensor** (QGT) is a fundamental object in quantum physics that encapsulates both the geometry and topology of quantum states in parameter space.

### Mathematical Definition

$$Q_{ij}^n(\mathbf{k}) = \sum_{m \neq n} \frac{\langle n | v_i | m \rangle \langle m | v_j | n \rangle}{(E_n - E_m)^2}$$

where $v_i = \partial_{k_i} H$ is the velocity operator.

### QGT Decomposition

The QGT naturally splits into two physically distinct parts:

| Component | Definition | Physical Meaning |
|-----------|------------|------------------|
| **Quantum Metric** | $g_{ij} = \text{Re}(Q_{ij})$ | Distance between quantum states in k-space |
| **Berry Curvature** | $\Omega_{ij} = -2 \cdot \text{Im}(Q_{ij})$ | "Magnetic field" in momentum space |

---

## Key Concepts

### 1. Quantum Metric (Real Part)

The quantum metric measures how much quantum states change as you move through the Brillouin zone. It sets a **geometric lower bound** on transport:

$$D \geq g_{ii} \cdot E_{gap}$$

Even in perfectly flat bands where conventional transport vanishes, the quantum metric ensures non-zero conductivity.

### 2. Berry Curvature (Imaginary Part)

The Berry curvature acts like a magnetic field in momentum space, causing:
- **Anomalous Hall Effect**: Transverse current without external B-field
- **Orbital Magnetization**: Intrinsic magnetic moment of Bloch electrons
- **Topological Invariants**: Chern number = $\frac{1}{2\pi} \int \Omega \, d^2k$

### 3. Effective Band Drude Weight

For flat-band systems like magic-angle TBG:

$$D = D_{conv} + D_{geom} = \text{(curvature)} + g \cdot E_{gap}$$

When $D_{conv} \approx 0$ (flat band), the geometric term provides metallic behavior!

---

## Application: Twisted Bilayer Graphene

At the **magic angle** (~1.1°), TBG exhibits:
- Nearly flat electronic bands
- Strong electron correlations
- Unconventional superconductivity
- **QGT-dominated transport**

The Quasi-QGT connects directly to experimental observables:
- **Real part** → Band Drude Weight (optical conductivity)
- **Imaginary part** → Orbital Angular Momentum (ARPES, dichroism)

---

## APIs Demonstrated

| API | Purpose |
|-----|---------|
| `quantum_geometric_tensor` | Computes QGT component $Q_{ij}^n$ |
| `effective_band_drude_weight` | Transport weight including geometric contribution |
| `QuantumEigenvector` | Bloch state wavefunctions |
| `QuantumVelocity` | Velocity operator matrix elements |
| `QuantumMetric` | Real part of QGT |

---

## References

- Kang et al., arXiv:2412.17809 - Experimental probe of Quasi-QGT
- Provost & Vallee (1980) - Original QGT formulation
- Xie et al., Nature (2021) - Spectroscopic signatures of QGT in TBG
