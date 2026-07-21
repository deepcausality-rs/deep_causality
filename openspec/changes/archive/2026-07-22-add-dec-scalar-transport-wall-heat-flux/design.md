## Context

The crate has two immersed-body paths and neither carries a wall heat flux. The QTT path is volume
penalization — no surface, so a flux is not well-posed there. The DEC path has real cut-cell geometry
(`CutFaceFragment` with area, outward normal and centroid) and already integrates a surface force over
it, but marches velocity only.

So the missing piece is narrow: a scalar field on the DEC path. Everything downstream of that is a
transcription of machinery that exists and is in use.

## Goals / Non-Goals

**Goals:**

- A scalar advected and diffused on the DEC manifold with the same operators the velocity uses.
- A Dirichlet wall temperature on the immersed body.
- `wall_heat_flux` as a genuine Fourier surface integral over cut-cell fragments.
- A verification harness gating it against a closed-form reference.

**Non-Goals:**

- **The momentum path.** The velocity rate, the projection and the existing surface forces are
  untouched. The scalar is passive: it reads the velocity and does not feed back.
- **Buoyancy or any thermal–momentum coupling.** A Boussinesq term is a separate feature; adding it
  here would make the scalar active and put the momentum path in scope.
- **Temperature-dependent properties.** `k` and `κ` are constants. Real `k(T)` belongs with the Gap-2
  reacting energy equation.
- **Replacing `penalization_heat_integral`.** It stays, with its meaning and its name. This adds the
  quantity it is not.
- **A QTT-side Fourier flux.** Argued against in the proposal: on a smeared mask the interface gradient
  scales inversely with the blur width.

## Decisions

### D1 — The scalar is a 0-cochain, advected by the interior product

`∂T/∂t = −i_u(dT) − κ·Δ_dR T`.

*Why:* it is the DEC-native statement of `∂T/∂t + u·∇T = κ∇²T`. For a 0-form `T`, `dT` is a 1-form and
`i_u(dT)` is the 0-form `u·∇T` — the same interior product the momentum rate uses for the Lamb vector,
at a different grade. Diffusion is `Δ_dR` on 0-forms, and the sign follows the crate's Stage-0 pin
(`Δ_dR = −∇²`), so `+κ∇²T` enters as `−κ·Δ_dR T`, exactly as `+ν∇²u` enters as `−ν·Δ_dR u♭`.

*The consequence worth stating:* a scalar and a velocity component are then differentiated by the same
code. A sign error in the shared operator moves both, which is what makes the momentum path's existing
verification partly cover the scalar too.

*Alternative considered.* A finite-difference scalar transport alongside the DEC operators was rejected
on the audit's own evidence: §4b records a gate that "tests a re-implementation, not the shipped
solver", and a parallel discretisation is how that arises.

### D2 — The wall condition is Dirichlet, from the same geometry as no-slip

`T` is pinned to `T_w` on the body, with the pinned set derived from the cut-cell registry that supplies
the momentum no-slip constraint.

*Why:* the thermal and mechanical boundaries must describe the same body, or the flux is computed
against a wall in a different place from the one the flow sees. Deriving both from one geometry makes
that structural rather than a convention two call sites must maintain.

### D3 — One-sided wall-normal gradient, not a central difference

`∂T/∂n ≈ (T_sample − T_w)/Δh`, with `T_w` anchored at the fragment centroid and `T_sample` taken one
wall-normal step into the fluid by multilinear interpolation.

*Why:* this is the reconstruction `viscous_surface_force` already uses, citing Kirkpatrick et al.
(2003), and the reasons transfer unchanged. A central difference straddling the cut mixes fluid and
solid-side nodes over a full cell; the one-sided form reads the gradient from the wall to the first
fluid sample over the actual perpendicular distance, so it is exact on a linear profile and far better
at a curved wall.

*Reusing the same reconstruction has a second benefit:* the two diagnostics agree about where the wall
is and how far the first sample sits from it, so a discrepancy between friction and heat flux is
physics rather than two different wall models.

### D4 — Verify against conduction first, convection second

The gate is a closed-form conduction case; a convective reference (a Nusselt correlation) is a later
addition.

*Why:* pure conduction isolates what this change introduces. It has an exact solution, needs no
turbulence model or correlation uncertainty, and fails loudly if the gradient reconstruction, the
fragment areas or the sign convention are wrong. A Nusselt correlation bundles the scalar operator,
the velocity field, the boundary layer and an empirical fit into one number — a poor first gate,
because it cannot say which of them moved. It is the right *second* gate.

*This follows the audit's rule directly:* a gate must be able to fail for a reason you can name.

### D5 — Sign convention stated at the API, not inferred

With `n` the body's outward normal, positive `q` is heat leaving the wall into the fluid.

*Why:* the audit's central finding is quantities whose name outruns their content. A flux whose sign
convention the caller has to infer from the source is the same defect in miniature, and for a TPS
consumer the sign is the difference between heating and cooling.

## Risks / Trade-offs

- **The scalar is passive, so a thermally-driven flow is out of reach.** → Accepted and stated as a
  Non-Goal. Buoyancy is a separate change; bundling it would put the momentum path in scope.
- **The flux is resolution-bound at the body**, as `viscous_surface_force` already documents itself to
  be. → Inherited, not new. The verification bound must be justified against resolution rather than
  pinned to whatever the first run produced.
- **Two heat quantities now exist with similar names.** → The spec requires each to state what the other
  is and why it is not interchangeable. This is the risk change 4 created by reserving the name; leaving
  it reserved and empty was not better.
- **Pinning `T` on the body changes no existing result**, since no shipped case marches a scalar on the
  DEC path. → Makes the change additive, and the momentum regression suite should be bit-identical.

## Migration Plan

Additive; no existing behaviour changes.

1. **Scalar rate** — advection + diffusion, verified against the analytic mode-decay and pure-advection
   cases before any wall is involved.
2. **Wall Dirichlet condition** — verified by the wall holding its value.
3. **`wall_heat_flux`** — the surface integral, verified by the isothermal-zero and sign-reversal cases.
4. **Verification harness** — the analytic conduction gate.

Steps 1–3 are independently revertible. The momentum suite must stay bit-identical throughout; if it
moves, something outside this change's scope was touched.

## Open Questions

- **Should `k` and `κ` be independent inputs, or related through `k = ρ·c_p·κ`?** Independent is
  simpler and avoids implying a `c_p` the crate does not carry; the relation can be documented so a
  caller supplying both consistently gets the physical answer.
- **Does the scalar belong in the marched state or alongside it?** Carrying it in the state makes it
  march with the velocity under one integrator; carrying it alongside keeps the momentum state
  bit-identical by construction. The latter is safer for this change and is what the Non-Goals imply.
