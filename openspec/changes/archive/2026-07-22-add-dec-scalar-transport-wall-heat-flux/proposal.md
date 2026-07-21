# DEC scalar transport and a fragment-resolved wall heat flux

## Why

The crate has no honest wall heat flux anywhere, and for a re-entry thermal-protection consumer that is
the safety-critical quantity — the audit's §4b says so explicitly.

Change 4 renamed the QTT observable to `penalization_heat_integral`, because
`(1/η)∫χ(T_w−T)dV` is a temperature-weighted volumetric rate, not a flux: no gradient, no conductivity,
no wall normal. That was the right fix for a misleading name, but it left the crate with a reserved name
and nothing behind it.

**The reserved name can now be filled, and the blocker is smaller than the change-4 design claimed.**
That design deferred a real Fourier-law flux to "the Gap-2 reacting energy equation". That is the wrong
dependency. Checking what actually exists:

| Part Fourier's law needs | Status |
|---|---|
| A wall surface with area and outward normal | ✅ `CutFaceFragment { area, outward_normal, centroid }`, shipped |
| A surface-integral diagnostic over those fragments | ✅ `viscous_surface_force` computes `∮ μ(∇u+∇uᵀ)·n dA` |
| A one-sided wall-normal gradient to the true surface distance | ✅ Kirkpatrick et al. (2003), already implemented there |
| A normal-consistency check `∮ n dA = 0` | ✅ `fragment_area_vector` |
| **A temperature field on the DEC path** | ❌ **absent — the DEC solver marches velocity only** |

So the geometry is real and in use; what is missing is a scalar to differentiate. Gap-2 would supply a
*better* temperature field (real `k(T)`, chemistry); it is not what unblocks a flux.

**Why not compute it on the QTT penalized path instead.** Volume penalization has no wall surface — only
a mask smoothed over `SMOOTH_CELLS·dx`. For the `tanh` mask the crate uses, `|∇χ|` peaks at `1/(2w)` and
`T` relaxes across the same width, so any interface gradient scales as `k·ΔT/w`, inversely with a purely
numerical parameter. The audit measured the *drag* — a volume integral, which averages — moving **6.1×**
across that sweep (§5b); a wall-normal derivative amplifies where an integral averages, so the flux would
be worse. Spectral differentiation does not rescue it: the mask transitions over ~2 grid points, near
Nyquist, so an FFT gradient rings at the interface rather than sharpening it. Computing the wall exchange
from the penalization source, which is what `penalization_heat_integral` does, is the standard
volume-penalization answer precisely because the interface gradient is unreliable on a smeared mask.

## What Changes

- **Scalar transport on the DEC path.** A temperature 0-cochain advected and diffused on the same
  manifold the velocity marches: `∂T/∂t = −i_u(dT) − κ·Δ_dR T`, using the interior product for advection
  and the Hodge–de Rham Laplacian for diffusion, with the same sign pin the viscous term uses.
- **A Dirichlet wall temperature** on the immersed body, so the scalar has a boundary condition to
  produce a gradient against.
- **`wall_heat_flux`** — the reserved name, filled: `q = −k ∮_S ∇T·n dA` over the cut-cell fragments,
  with the same one-sided wall-normal reconstruction `viscous_surface_force` uses.
- **A verification harness** gating the flux against an analytic conduction reference.

## Impact

- Affected specs: `dec-scalar-transport` (new), `dec-surface-diagnostics` (new)
- Affected code: `solvers/dec/` — a new scalar rate, a wall-temperature constraint, and an addition to
  `surface_force.rs`; a new `verification/` harness
- **No change to the momentum path.** The velocity rate, the projection and the existing surface forces
  are untouched; the scalar rides on the same manifold and operators.
- `penalization_heat_integral` keeps its name and meaning — this adds the quantity it is not, rather
  than replacing it.
