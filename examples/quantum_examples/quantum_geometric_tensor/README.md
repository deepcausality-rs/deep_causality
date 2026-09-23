# The Quantum Geometric Tensor, and the Transport It Forces

This example computes the quantum geometric tensor of a flat band and feeds its metric into the
Drude weight. A flat band has no dispersion, so conventional band theory says it cannot conduct, yet
magic-angle twisted bilayer graphene has flat bands and conducts. The missing term is geometric.

```bash
cargo run -p quantum_examples --example quantum_geometric_tensor
```

## The problem

Band energies leave out the *states*: how much the wavefunction turns as momentum moves across the
Brillouin zone. The **quantum geometric tensor** carries that information:

```text
Q_ij = Σ_{m ≠ n}  ⟨n|v_i|m⟩ ⟨m|v_j|n⟩ / (E_n − E_m)²
```

where `v_i = ∂H/∂k_i` is the velocity operator. It gives one complex number per pair of axes, and
its two parts describe different physics:

| Part | Definition | Symmetry | Meaning |
|---|---|---|---|
| Quantum metric | `g_ij = Re(Q_ij)` | symmetric | how far apart two neighbouring states are |
| Berry curvature | `Ω_ij = −2 Im(Q_ij)` | antisymmetric | a magnetic field in momentum space |

The Hermitian structure forces the symmetry: a Hermitian tensor decomposes into a real symmetric
part and an imaginary antisymmetric part, so `Ω_xx` is zero for the same reason the cross product of
a vector with itself is.

## From geometry to a current

Conventional transport comes from band curvature, the inverse effective mass. The Drude weight the
library computes adds the geometric term to it:

```text
D = (D_conv + g̃_xx · E_gap) · a²
```

With a perfectly flat band `D_conv = 0`, so the entire weight is geometric: the **geometric lower
bound** on conductivity.

The run computes `g_xx` from the model and feeds that value into the formula, so the transport kernel
receives the number the tensor produced.

### The step that is easy to skip

`Q_ij` comes out in **nm²**: the velocity matrix elements carry energy × length, so the quantum
metric is an area. The transport kernel expects the metric in units of the cell:

```text
g̃_xx = g_xx / a²
```

Passing `g_xx` directly gives the kernel an area where it expects a ratio, and since the kernel
then multiplies the whole weight by `a²`, the answer is off by the cell area twice over.
`model::reduced_metric` performs that division.

## What the code demonstrates

| Operation | Carries |
|---|---|
| `fmap` | one axis pair → the QGT component for that pair, which may fail |
| `sequence` | a tensor of fallible components → one fallible tensor |
| `fmap` | `Q → Re(Q)` and `Q → −2 Im(Q)`, the two readings of one object |
| `fold` | the diagonal of `g` → its trace |

**`sequence` surfaces every failure.** `fmap` leaves a tensor whose every cell might have failed.
One call turns that inside out, so a single `?` covers all four components and the `[2, 2]` shape
survives the traversal. A component that carried no value stops the run instead of vanishing from
the output.

## Units

One system throughout: **energies in meV, lengths in nm, velocity matrix elements in meV·nm**. Mixed
units break the metric-to-transport step silently; the header prints the system.

Decimals such as `0.246` are derived from integers at the working precision instead of written as
literals. A literal is an `f64` value rounded once and then widened; deriving from integers rounds
only at the precision in force.

## Output

```text
Bands
  band 0 (flat)        1.000 meV
  band 1 (remote)     10.000 meV
  gap                  9.000 meV
  lattice constant     0.246 nm

Q_ij for the flat band, and its decomposition
  pair      Re(Q_ij)      Im(Q_ij)      g_ij          Omega_ij
                                        (nm^2)        (nm^2)
  Q_xx       0.004198      0.000000      0.004198      0.000000
  Q_xy       0.000000     -0.004198      0.000000      0.008395
  Q_yx       0.000000      0.004198      0.000000     -0.008395
  Q_yy       0.004198      0.000000      0.004198      0.000000

The symmetry the decomposition forces
  g_xy - g_yx                 0.000e0   the metric is symmetric
  Omega_xy + Omega_yx         0.000e0   the curvature is antisymmetric
  Omega_xx                    0.000e0   so the diagonal of it vanishes

From geometry to transport
  tr g = g_xx + g_yy            0.008395 nm^2
  g_xx                          0.004198 nm^2
  g~_xx = g_xx / a^2            0.069362        dimensionless
  E_gap                         9.000000 meV

  D = (D_conv + g~_xx * E_gap) * a^2
    flat band alone             0.000000 meV*nm^2
    with the geometry           0.037778 meV*nm^2
```

`g_xx = 0.34 / 9² = 0.004198 nm²` follows from the one non-zero matrix element,
`⟨0|v_x|1⟩ = 0.5 + 0.3i`, and the 9 meV gap. The metric is diagonal and the curvature purely
off-diagonal because of the relative phase between `v_x` and `v_y`: equal phases would leave `Q`
real and the Berry curvature zero everywhere.

## Precision is a parameter

```rust
pub type FloatType = Float106;
```

Every constant is declared at that type through `const_scalar_from_int!`, so no conversion runs at
any call site. The alias is `Float106` so that a hard-coded `f64` fails to compile; with the alias
at `f64` it would go unnoticed. All four scalars run, and `BFloat16` lands `g̃_xx` at `0.068848`
against `0.069362`, the error of its eight-bit mantissa.

## What this example covers

The example states band geometry as a composition over the library's types; precision as a
parameter and categorical composition follow from that form. The tensor is one object with two
readings, and the real one forces a current the energies alone do not account for. Everything else
stays simple: two bands at a single **k**-point, eigenstates as the basis, velocity operators written
down instead of differentiated from a Hamiltonian, and no integration over the Brillouin zone.

A calculation an experimentalist would compare against adds the parts this one leaves out: a
tight-binding or continuum Hamiltonian for real TBG at the magic angle, `v_i` obtained by
differentiating it, a **k**-mesh with the tensor computed at every point, and the Chern number and
orbital magnetisation as integrals of `Ω` over that mesh. The moiré lattice constant is ~13 nm
against graphene's 0.246 nm, which shifts the scales.

## How to grow the example toward a band-structure calculation

Each step keeps the structure already here.

- **A k-mesh.** Make the tensor `[n_k, 2, 2]` instead of `[2, 2]`. `fmap` and `sequence` already
  have that shape; only the pair list grows.
- **Chern number.** `fold` the `Ω_xy` cells over the mesh and divide by `2π`. The reduction is the
  one already in `main`, with a different closure.
- **A real Hamiltonian.** Replace `model::two_band_model` with a tight-binding `H(k)` and obtain
  `v_i = ∂H/∂k_i` by automatic differentiation through `deep_causality_calculus` instead of writing
  the velocity operators by hand.
- **More bands.** `NUM_BANDS` is a constant, and the kernel already sums over every `m ≠ n`.

## References

- Provost & Vallée (1980) — the original QGT formulation
- Kang et al., arXiv:2412.17809 — experimental probe of the quasi-QGT
- Xie et al., *Nature* (2021) — spectroscopic signatures of quantum geometry in TBG

## Files

| File | Holds |
|---|---|
| `main.rs` | the alias, the tensor, its decomposition, and the chain into transport |
| `model.rs` | the units, the two-band model, one QGT component, and the reduction by `a²` |
| `utils_print.rs` | the presentation, and the only `lower` calls |
