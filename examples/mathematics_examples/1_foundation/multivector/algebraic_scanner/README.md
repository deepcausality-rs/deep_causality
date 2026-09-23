# Algebraic Scanner: Automated Theory Search

This example searches Clifford algebras for the dimensions that admit a complex structure (I² = -1).

## How to Run

```bash
cargo run -p mathematics_examples --example algebraic_scanner_examples
```

---

## Engineering Value

An algebra with a complex structure can stand in for the complex numbers, which matters for:
- **Quantum Mechanics**: wavefunctions are complex-valued
- **Supersymmetry**: requires specific Clifford algebra properties
- **Signal Processing**: Fourier analysis runs on a complex structure

The scan replaces a tedious manual calculation.

---

## What It Does

Scans Clifford algebras Cl(p,q), Euclidean and Minkowski, from dimension 1 to 9 and checks whether the pseudoscalar I satisfies:

```
I² = -1  (Complex Structure)
```

### Pseudoscalar

The pseudoscalar I is the highest-grade element of the algebra, the product of all basis vectors. Its square depends on the metric signature:
- **Euclidean Cl(n,0)**: I² = ±1 depending on dimension
- **Minkowski Cl(p,q)**: I² depends on signature

---

## Output Interpretation

```
[MATCH] Dimension 3: Euclidean signature Cl(3, 0) admits Complex Structure. I² = -1.0000
```

Cl(3,0), the algebra of 3D Euclidean space, has a complex structure; the quaternions used for 3D rotations are related to Cl(3,0).

---

## Adapting This Example

1. **Change dimension range**: Modify `MAX_DIM` (memory grows as 2^n)
2. **Add custom signatures**: Use `Metric::Custom` for arbitrary (p,q) signatures
3. **Check other properties**: Extend to check for division algebra structure, etc.

---

## Key APIs Used

- `CausalMultiVector::new()` - Create multivector with metric
- `geometric_product()` - Compute I²
- `Metric::Euclidean(n)`, `Metric::Minkowski(n)` - Signature selection
