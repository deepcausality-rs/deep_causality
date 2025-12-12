# Algebraic Scanner: Automated Theory Search

This example demonstrates an automated search through Clifford Algebras to find dimensions that admit a complex structure (I² = -1).

## How to Run

```bash
cargo run -p physics_examples --example algebraic_scanner
```

---

## Engineering Value

In theoretical physics, finding algebras with complex structure is crucial for:
- **Quantum Mechanics**: Complex numbers are fundamental (wavefunctions)
- **Supersymmetry**: Requires specific Clifford algebra properties
- **Signal Processing**: Complex structure enables Fourier analysis

This example automates what would otherwise be tedious manual calculation.

---

## What It Does

Scans Clifford Algebras Cl(p,q) from dimension 1 to 9 and checks if the pseudoscalar I satisfies:

```
I² = -1  (Complex Structure)
```

### Pseudoscalar

The pseudoscalar I is the highest-grade element of the algebra (product of all basis vectors). Its square depends on the metric signature:
- **Euclidean Cl(n,0)**: I² = ±1 depending on dimension
- **Minkowski Cl(p,q)**: I² depends on signature

---

## Output Interpretation

```
[MATCH] Dimension 3: Euclidean signature Cl(3, 0) admits Complex Structure. I² = -1.0000
```

This means Cl(3,0) - the algebra of 3D Euclidean space - has a complex structure, which is why quaternions (related to Cl(3,0)) are so useful in 3D rotations.

---

## Adapting This Example

1. **Change dimension range**: Modify `max_dim` (note: memory grows as 2^n)
2. **Add custom signatures**: Use `Metric::Custom` for arbitrary (p,q) signatures
3. **Check other properties**: Extend to check for division algebra structure, etc.

---

## Key APIs Used

- `CausalMultiVector::new()` - Create multivector with metric
- `geometric_product()` - Compute I²
- `Metric::Euclidean(n)`, `Metric::Minkowski(n)` - Signature selection
