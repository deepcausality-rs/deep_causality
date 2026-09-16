# The Quantum Geometric Tensor, and the Transport It Forces

A flat band has no dispersion, so conventional band theory says it cannot conduct. Magic-angle
twisted bilayer graphene has flat bands and conducts anyway. The missing term is geometric, and this
example computes it and hands it to transport.

```bash
cargo run -p quantum_examples --example quantum_geometric_tensor
```

## The problem

A band is usually described by its energies. That leaves out everything about the *states*: how much
the wavefunction itself turns as momentum moves across the Brillouin zone. The **quantum geometric
tensor** carries exactly that:

```text
Q_ij = Σ_{m ≠ n}  ⟨n|v_i|m⟩ ⟨m|v_j|n⟩ / (E_n − E_m)²
```

where `v_i = ∂H/∂k_i` is the velocity operator. It is one complex number per pair of axes, and its
two parts are two different pieces of physics:

| Part | Definition | Symmetry | Meaning |
|---|---|---|---|
| Quantum metric | `g_ij = Re(Q_ij)` | symmetric | how far apart two neighbouring states are |
| Berry curvature | `Ω_ij = −2 Im(Q_ij)` | antisymmetric | a magnetic field in momentum space |

The symmetry is forced rather than imposed. A real symmetric part and an imaginary antisymmetric
part is what a Hermitian tensor decomposes into, so `Ω_xx` is zero for the same reason a
cross-product of a vector with itself is.

## From geometry to a current

Conventional transport comes from band curvature, the inverse effective mass. The Drude weight the
library computes adds the geometric term to it:

```text
D = (D_conv + g̃_xx · E_gap) · a²
```

With a perfectly flat band `D_conv = 0`, so the entire weight is geometric. That is the **geometric
lower bound** on conductivity.

The run computes `g_xx` from the model and feeds it into that formula. The two halves are one chain:
the number that reaches the transport kernel is the number the tensor produced, not a stand-in.

### The step that is easy to skip

`Q_ij` comes out in **nm²** — the velocity matrix elements carry energy × length, so the quantum
metric is an area. The transport kernel wants the metric in units of the cell:

```text
g̃_xx = g_xx / a²
```

Handing the kernel `g_xx` directly gives it an area where it expects a ratio, and since the kernel
then multiplies the whole weight by `a²`, the answer comes out wrong by the cell area twice over.
`model::reduced_metric` is that one division, and it is the reason the units line up.

## What the code demonstrates

| Operation | Carries |
|---|---|
| `fmap` | one axis pair → the QGT component for that pair, which may fail |
| `sequence` | a tensor of fallible components → one fallible tensor |
| `fmap` | `Q → Re(Q)` and `Q → −2 Im(Q)`, the two readings of one object |
| `fold` | the diagonal of `g` → its trace |

**`sequence` is what makes the failure honest.** `fmap` leaves a tensor whose every cell might have
failed. Turning that inside out is one call, so a single `?` covers all four components, and the
`[2, 2]` shape survives the traversal. A component that carried no value stops the run rather than
quietly vanishing from the output.

## Units

One system throughout: **energies in meV, lengths in nm, velocity matrix elements in meV·nm**. Mixed
units are what make the metric-to-transport step go wrong silently, so there is only one system here
and the header prints it.

Decimals such as `0.246` are derived from integers at the working precision rather than written as
literals. A literal is an `f64` value rounded once and then widened, and the point of the alias is
that nothing is rounded at a precision other than the one in force.

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
`⟨0|v_x|1⟩ = 0.5 + 0.3i`, and the 9 meV gap. The metric is diagonal and the curvature is purely
off-diagonal, which is what the relative phase between `v_x` and `v_y` puts there: equal phases would
leave `Q` real and the Berry curvature zero everywhere.

## Precision is a parameter

```rust
pub type FloatType = Float106;
```

Every constant is declared at that type through `const_scalar_from_int!`, so no conversion runs at
any call site. It sits at `Float106` rather than `f64` on purpose: a hard-coded `f64` is invisible
while the alias *is* `f64`, and a compile error the moment the two differ. All four scalars run, and
`BFloat16` lands `g̃_xx` at `0.068848` against `0.069362`, which is the eight-bit mantissa showing
through.

## What this example covers

The goal is to reformulate the essence of band geometry as a composition over the library's types,
and to get precision as a parameter and categorical composition for free once it is in that form.
The essence is that the tensor is one object with two readings, and that the real one forces a
current the energies alone do not account for. The model keeps that and holds everything else
simple: two bands at a single **k**-point, eigenstates taken as the basis, velocity operators
written down rather than differentiated from a Hamiltonian, and no integration over the Brillouin
zone.

A calculation an experimentalist would compare against adds the parts this one leaves out: a
tight-binding or continuum Hamiltonian for real TBG at the magic angle, `v_i` obtained by
differentiating it, a **k**-mesh with the tensor computed at every point, and the Chern number and
orbital magnetisation as integrals of `Ω` over that mesh. The moiré lattice constant is also ~13 nm
rather than graphene's 0.246 nm, which moves the scales considerably.

## How to grow the example toward a band-structure calculation

Each step keeps the structure already here.

- **A k-mesh.** Make the tensor `[n_k, 2, 2]` instead of `[2, 2]`. `fmap` and `sequence` already
  have that shape; only the pair list grows.
- **Chern number.** `fold` the `Ω_xy` cells over the mesh and divide by `2π`. The reduction is the
  one already in `main`, with a different closure.
- **A real Hamiltonian.** Replace `model::two_band_model` with a tight-binding `H(k)` and obtain
  `v_i = ∂H/∂k_i` by automatic differentiation through `deep_causality_calculus`, so the velocity
  operators stop being written down by hand.
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
