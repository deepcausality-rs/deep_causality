## ADDED Requirements

### Requirement: Fourier-law wall heat flux over cut-cell fragments

The crate SHALL expose `wall_heat_flux` computing `q = −k ∮_S ∇T·n dA` over the immersed body's
cut-cell fragments. This extends the surface-integral diagnostic beyond forces: the capability's
subject is a read-only quantity integrated over a body's `CutFaceFragment`s, and a heat flux is such
a quantity even though it is not a force. It SHALL share the fragment iteration and the wall-normal
reconstruction with the viscous traction rather than reimplementing them, so the two diagnostics
cannot disagree about where the wall is.

The flux is defined with `n` each fragment's outward unit normal, `dA` its area, and `k` the thermal
conductivity. This is Fourier's law as an actual surface integral, and the name SHALL be used for no
other quantity.

The wall-normal derivative SHALL be reconstructed **one-sided to the true surface distance**, as the
viscous surface force already does (Kirkpatrick et al. 2003): the wall value `T_w` is anchored at the
fragment centroid, the field is sampled one wall-normal step `Δh` into the fluid by multilinear
interpolation, and `∂T/∂n ≈ (T_sample − T_w)/Δh`. A central difference straddling the cut SHALL NOT be
used, since it mixes fluid and solid-side nodes across a full cell.

The sign convention SHALL be stated: with `n` the outward normal of the **body**, a positive `q` is heat
flowing from the wall into the fluid.

#### Scenario: The flux is a surface integral, not a volume integral
- **WHEN** the flux is computed for a body
- **THEN** it is accumulated over fragment areas with fragment normals, and its dimensions are those of
  `k·[T]·[L]^(D−2)` — a per-area quantity integrated over the wetted surface

#### Scenario: An isothermal field carries no flux
- **WHEN** the fluid is at the wall temperature everywhere
- **THEN** the reported flux is zero to round-off, since `∇T·n` vanishes at every fragment

#### Scenario: The flux reverses with the temperature difference
- **WHEN** the sign of `T_w − T_fluid` is reversed
- **THEN** the reported flux reverses sign and preserves magnitude

#### Scenario: The flux matches an analytic conduction reference
- **WHEN** a configuration with a closed-form conduction solution is marched to steady state
- **THEN** the computed flux matches the analytic value within a stated, resolution-justified bound

#### Scenario: The flux is distinguished from the penalization heat integral
- **WHEN** a consumer reads either quantity's documentation
- **THEN** `wall_heat_flux` is identified as a Fourier surface flux on the cut-cell path and
  `penalization_heat_integral` as a volumetric penalization rate on the QTT path, with the reason the
  latter is not the former stated rather than implied
