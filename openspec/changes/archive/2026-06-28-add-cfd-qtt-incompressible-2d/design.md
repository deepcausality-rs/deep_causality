## Context

`add-cfd-qtt-tensor-bridge` delivered the 1-D foundation: a QTT codec, hand-built shift MPOs, FD stencil
assembly via the operator algebra, and a 1-D linear `Marcher` (`QttLinear1d`). This change lifts that to
a **2-D incompressible Navier–Stokes** solver — the projection + nonlinearity that make it a real
flowfield — following Peddinti et al. (MPS incompressible NS) and using `solve::linear` (AMEn) for the
pressure Poisson, all on `deep_causality_tensor`.

## Goals / Non-Goals

Goals: 2-D QTT encoding, lifted axis operators, a tensor-train pressure projection (AMEn Poisson + Leray),
and a periodic 2-D incompressible NS marcher with nonlinear convection — validated against the analytic
Taylor–Green vortex, with bounded rank.

Non-Goals (next change): CfdFlow wiring, observables, immersed BCs, Gap-2 ionization physics, 3-D.

## Decisions

- **2-D layout (serial).** A `2^Lx × 2^Ly` field reshapes to `[2; Lx, 2; Ly]` (x-modes then y-modes) and
  `from_dense` gives the MPS. Per-axis serial blocks (not interleaved) keep the axis operators trivial to
  build; interleaving is a later optimization if cross-axis structure needs lower bonds.

- **Axis operators by lifting.** `∂ₓ = (1-D shift stencil on the Lx x-modes) ⊗ I` on the Ly y-modes —
  built by concatenating the 1-D operator's cores with `Ly` identity cores (the bond-1 boundary between
  the blocks joins them). `gradient_y` lifts on the leading x-block instead. `laplacian_2d =
  lift_x(lap_1d) + lift_y(lap_1d)`, summed and rounded via the operator algebra. A `lift` helper on the
  MPO (concatenate identity cores) is the one new operator primitive; everything else reuses
  `add`/`sub`/`scale`/`compose`/`round`.

- **Pressure projection — spectral, not iterative (ARIZ resolution).** The projection needs `∇p`, not
  `p`; the periodic Laplacian's null space is the *constant*, whose gradient is **zero**, so `∇p` is
  unique despite the singular operator — the singularity is a non-problem. On a periodic grid the
  Laplacian is **diagonal in the Fourier basis** (`λ_k = −(2−2cos(2πk/N))/Δx²`), so the Poisson solve is
  *exact*: `p̂_k = div̂_k/λ_k` for `k≠0`, `p̂_0 = 0` — which pins the null space by construction, with no
  regularization and no iteration. Implementation: Tier-A small grids dequantize → eigen-solve →
  requantize (trivially correct); the scalable form is a **QFT-MPO** keeping it in QTT.
  `solve::linear` (AMEn) is retained for the *future* wall-bounded (non-diagonal) case, not used here.
  Then `u = u* − gradient_x·p`, `v = v* − gradient_y·p` (apply + sub + round) — the discrete Leray
  projection.

- **Nonlinear convection — round-as-you-build (ARIZ resolution).** `u·∇u = u ⊙ (∂ₓu) + v ⊙ (∂ᵧu)` via the
  **fused `hadamard_rounded`** (already in the tensor crate), which compresses each squared-bond core as
  it builds, so the `r²` blow-up is never materialized — the rank growth is spurious and `round` reveals
  the true (physical) rank. The round tolerance is tied to the discretization error (no point resolving
  below the spatial-truncation floor) with a `max_bond` backstop. Diffusion (`ν·∇²`) physically damps the
  small scales that carry the high rank, so for resolved flow the rank self-limits. TT-cross
  (`apply_nonlinear`) is the escape hatch — it builds the term *at* a capped rank — if Hadamard+round is
  insufficient.

- **Marcher.** `QttIncompressible2d` implements `Marcher` with `State = (CausalTensorTrain, CausalTensorTrain)`
  for `(u, v)`. Step: form `rate = −(u·∇)u + ν·∇²u` per component, `u* = round(u + Δt·rate)`, then
  `(u, v) = project(u*, v*)`. Explicit time stepping with per-operation rounding (same rationale as
  `QttLinear1d` — Rk4's non-rounding stages would blow up the rank).

- **Module placement.** Extends `tensor_bridge/` (2-D codec, lifts, projection) and `solvers/qtt/`
  (`QttIncompressible2d` beside `QttLinear1d`).

## Risks / Trade-offs

Both headline risks were worked through with ARIZ (`ctx/ariz-template.txt`) before implementation and
**resolved by reformulation**, not deferred:

- **Singular periodic Poisson — RESOLVED.** The projection needs `∇p` (null-space-invariant), not `p`, and
  the periodic Laplacian is diagonal in Fourier space, so the solve is *exact* with the `k=0` mode zeroed
  (null space pinned by construction). No iterative AMEn, no regularization, no convergence risk for the
  periodic case. Residual risk is only the QFT-MPO construction effort (scalable form); the dequantize-FFT
  interim is trivially correct at Tier-A sizes. **[resolved; AMEn deferred to the wall-bounded case]**
- **Nonlinear rank growth — CONTROLLED.** The `r²` Hadamard blow-up is spurious (the physical rank is low
  for resolved flow) and transient (the fused `hadamard_rounded` never materializes it); diffusion damps
  the high-rank small scales; the round tolerance is tied to the discretization floor with a `max_bond`
  cap. The validation sweeps error/bond to confirm the plateau. The irreducible part — unbounded rank at
  high-Re/turbulence — is fenced to **Tier-B** (out of scope), with TT-cross as the in-scope escape hatch.
  **[controlled for resolved flow; turbulence is Tier-B]**
- **Power-of-two grids per axis.** A real constraint, guarded.
- **Scope discipline.** Periodic, 2-D, no walls/observables. Immersed BCs and wiring are deferred.
