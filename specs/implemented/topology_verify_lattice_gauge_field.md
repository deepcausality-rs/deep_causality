# Lattice Gauge Field Verification Specification

## Overview

This specification outlines the approach for validating the `LatticeGaugeField` implementation in
`deep_causality_topology` by implementing known lattice gauge theory calculations and comparing results against
published reference values.

## Verification Strategy

The verification follows the same pattern used for the continuous `GaugeField`:

1. Implement calculations for a well-understood physical theory
2. Compute known observables
3. Compare against analytically derived or published reference values

---

## Phase 1: 2D U(1) Exact Solution (Recommended First)

### Why This Theory?

The 2D U(1) lattice gauge theory is **exactly solvable**, making it ideal for initial validation:

- Closed-form analytic solutions exist
- No Monte Carlo thermalization required
- Your implementation already supports U(1) and 2D lattices

### Observable: Average Plaquette

**Exact Formula:**
$$\langle P \rangle = \frac{I_1(\beta)}{I_0(\beta)}$$

where $I_n$ are modified Bessel functions of the first kind.

### Primary Reference

> **M. Creutz, *Quarks, Gluons and Lattices*, Cambridge University Press (1983), Chapter 8**
>
> The exact solution is derived from the fact that in 2D, the path integral factorizes over plaquettes. Each plaquette
> integral gives a ratio of Bessel functions. This is not a "published table" but an **exact analytic result** that can be
> computed to arbitrary precision.

The derivation uses:
$$Z = \prod_{\text{plaquettes}} \int_0^{2\pi} \frac{d\theta}{2\pi} e^{\beta \cos\theta} = \prod_p I_0(\beta)$$

and the plaquette expectation value follows from $\langle P \rangle = \partial \ln Z / \partial \beta$.

### High-Precision Reference Values (for DoubleFloat 106-bit testing)

These values are computed from the **exact formula** to 32 significant digits:

| β    | ⟨P⟩ = I₁(β) / I₀(β)                |
|------|------------------------------------|
| 0.5  | 0.24226845767486894622004965096203 |
| 1.0  | 0.44629221067969540862867127655328 |
| 2.0  | 0.69775705397737959322204756994581 |
| 3.0  | 0.80720842309070252574604451047186 |
| 4.0  | 0.86379295629099922539571839785098 |
| 5.0  | 0.89382968853082515877168223779606 |
| 6.0  | 0.91158548063408757619419088298917 |
| 8.0  | 0.93389598028088200055108648131458 |
| 10.0 | 0.94723257050594106316440854296207 |
| 20.0 | 0.97360397034839892188277086346506 |

> [!NOTE]
> Since the formula is exact, you can compute reference values to **any precision** by evaluating the modified Bessel
> functions $I_0(\beta)$ and $I_1(\beta)$ using your DoubleFloat type directly. The "reference values" above are derived
> values, not published experimental measurements.

### Computing Your Own Reference Values

For maximum precision verification, compute the Bessel functions directly using the series expansion:

$$I_n(x) = \sum_{k=0}^{\infty} \frac{1}{k! \, \Gamma(n+k+1)} \left(\frac{x}{2}\right)^{n+2k}$$

Or use the integral representation:
$$I_n(x) = \frac{1}{\pi} \int_0^{\pi} e^{x \cos\theta} \cos(n\theta) \, d\theta$$

For implementation, you can use high-precision libraries like `mpfr` or compute the series to enough terms for 106-bit
precision (typically ~50 terms suffice for β ≤ 20).

### Implementation Location

```
examples/physics_examples/lattice_u1_2d/
├── main.rs           # Calculate and verify ⟨P⟩
└── README.md         # Theory background and results
```

### Test Approach

```rust
// Pseudo-code for verification
fn verify_2d_u1_plaquette(beta: f64) -> bool {
    let lattice = Lattice::new([16, 16], [true, true]);
    let field = LatticeGaugeField::<U1, 2, f64>::identity(lattice, beta);

    let computed = field.try_average_plaquette().unwrap();
    let exact = bessel_i1(beta) / bessel_i0(beta);

    (computed - exact).abs() < 1e-10
}
```

---

## Phase 2: Strong Coupling Expansion

### Observable: Plaquette Expectation Value

For small β (strong coupling), the plaquette has a known series expansion:

$$\frac{\langle U_P \rangle}{N} \approx \frac{\beta}{2N^2} + O(\beta^2)$$

### Test Values

For U(1) (N=1) at β = 0.1:

- Expected: ≈ 0.05 + O(β²)

### Implementation

Add test cases in existing test file:

- `deep_causality_topology/tests/types/gauge/gauge_field_lattice/`

---

## Phase 3: Gradient Flow t₀ Scale

### Why This Observable?

The `t₀` scale is a standard lattice QCD reference scale, defined by:

$$t_0^2 \langle E(t_0) \rangle = 0.3$$

Your implementation already has `try_find_t0()` which implements this.

### Published Reference Values (Pure Gauge SU(3))

| Source               | √t₀ / a                                   |
|----------------------|-------------------------------------------|
| HotQCD Collaboration | 0.14229(98) fm                            |
| FLAG average         | Similar values for various lattice setups |

### Verification Approach

1. Create an SU(3) lattice with known β
2. Thermalize via Monte Carlo sweeps
3. Measure t₀ using gradient flow
4. Compare to published values at same β

### Implementation Location

```
examples/physics_examples/lattice_qcd_scale/
├── main.rs           # t₀ scale determination
└── README.md         # Comparison to FLAG values
```

---

## Phase 4: Wilson Loops and String Tension

### Observable: Creutz Ratio

The Creutz ratio extracts the string tension σ:

$$\chi(R,T) = -\ln\frac{W(R,T) \cdot W(R-1,T-1)}{W(R,T-1) \cdot W(R-1,T)}$$

As T → ∞: χ(R,T) → σa²

### Reference Value

- QCD string tension: σ ≈ 0.225 GeV² (√σ ≈ 445 MeV)

### Verification Approach

1. Compute Wilson loops via `try_wilson_loop()`
2. Calculate Creutz ratios at large R, T
3. Compare extracted σ to known value

---

## Phase 5: Polyakov Loop Phase Transition

### Observable: Polyakov Loop Order Parameter

$$P(\vec{x}) = \text{Tr}\left[\prod_{t=0}^{N_t-1} U_0(\vec{x}, t)\right]$$

### Physics

- **Confined phase:** ⟨P⟩ = 0 (infinite free quark energy)
- **Deconfined phase:** ⟨P⟩ ≠ 0 (free quarks possible)

### Reference Values

| Theory       | Lattice | Critical βc |
|--------------|---------|-------------|
| SU(3) Wilson | 4×Ns³   | ≈ 5.69      |
| SU(2) Wilson | 4×Ns³   | ≈ 2.30      |

### Verification

Use `try_average_polyakov_loop()` to observe transition near critical β.

---

## Implementation Summary

| Phase | Theory   | Observable | Method          | Difficulty |
|-------|----------|------------|-----------------|------------|
| 1     | 2D U(1)  | ⟨P⟩        | Exact           | ⭐ Easy     |
| 2     | Any      | ⟨P⟩        | Strong coupling | ⭐ Easy     |
| 3     | 4D SU(3) | t₀         | Gradient flow   | ⭐⭐ Medium  |
| 4     | 4D SU(3) | σ          | Wilson loops    | ⭐⭐⭐ Hard   |
| 5     | 4D SU(N) | ⟨P⟩        | Monte Carlo     | ⭐⭐⭐ Hard   |

---

## Recommended Implementation Order

### Immediate (Minimal Code)

1. **2D U(1) exact solution** - Identity field verification already passes. Add Monte Carlo thermalization and compare
   to Bessel function formula.

### Short-term

2. **Strong coupling tests** - Add test cases for small β limit.
3. **Improved action verification** - Verify Symanzik/Iwasaki coefficients match known values.

### Medium-term

4. **Gradient flow t₀ scale** - Requires thermalized SU(3) configurations.
5. **String tension extraction** - Requires larger lattices and statistics.

---

## Files to Create

```
examples/physics_examples/
├── lattice_u1_2d/
│   ├── Cargo.toml
│   ├── main.rs           # 2D U(1) verification
│   └── README.md
└── lattice_qcd_scale/    # Future: t₀ scale
    ├── Cargo.toml
    ├── main.rs
    └── README.md

deep_causality_topology/tests/types/gauge/gauge_field_lattice/
└── physics_benchmarks_tests.rs  # Strong coupling, coefficient checks
```

---

## References

1. M. Creutz, *Quarks, Gluons and Lattices*, Cambridge (1983) - Chapter on exact solutions
2. FLAG Lattice Averaging Group - t₀, w₀ scale values
3. HotQCD Collaboration - arXiv:1407.6387 - Gradient flow scales
4. Montvay & Münster, *Quantum Fields on the Lattice*, Cambridge (1994)
