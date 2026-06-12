## Context

After `add-fft` (archived 2026-06-12), the periodic DEC Navier–Stokes step
at 32³ f64 measures 137 ms serial / 57 ms parallel, down from 388 ms. The
profile is now: rate assembly 30 ms serial / 11.2 ms parallel per
evaluation × 4 RK4 stages ≈ 90 % of the step; the spectral Leray projection
is 1.9 ms. The rate is assembled by composing the generic DEC operators —
CSR matvecs for `d`/`δ` (column indices loaded per nonzero), per-cell
arithmetic for interior product/sharp, one materialized `CausalTensor` per
operator call. On a cubical lattice all of this structure is fixed at
manifold construction.

Stage 3 (walls) is specified by `cfd-gap.md` §G5: boundary-corrected Hodge
star, a Neumann–Poisson projection path, no-slip Laplacian rows, validated
Poiseuille-first then Ghia cavity. The G5 sizing note (≈ G1+G2+G3 combined)
is about the star and the boundary rows, not the solver wiring.

Constraints carried over: zero external runtime dependencies (Rayon
optional via the established `parallel` pattern), `unsafe_code = "forbid"`,
precision-generic over `RealField` (f32/f64/Float106), static dispatch, the
generic operator path stays as the cross-validation reference (the Stage 0
discipline), and specs/tests gate every behavioral claim.

## Goals / Non-Goals

**Goals:**

- Rate assembly ≥ 2× faster serial at 32³ (target ≤ 15 ms per evaluation),
  measured by the existing `dec_solver_benchmark`; 64³ full step parallel
  in the ~150–250 ms range, putting the Re-1600 dissipation curve at
  64³–128³ (Stage 1's open exit artifact) in overnight reach.
- Bit-for-bit unchanged *mathematics*: the stencil pipeline is an
  evaluation strategy for the same operators, equivalence-gated against
  the generic composition at every precision.
- Wall-bounded incompressible flow on cubical lattices: correct boundary
  Hodge star, no-flux pressure projection, no-slip viscous rows, validated
  against an exact solution (Poiseuille) before reference data (Ghia).
- The DCT direct-solve advantage on uniform boxes, so the cavity's
  pressure solve is O(N log N) like the torus path.

**Non-Goals:**

- IMEX / integrating-factor time integration (changes the march's
  semantics; revisit after the spectral-diffusion drop-in is validated).
- A pseudo-spectral solver path (Fourier collocation for advection). The
  DEC composition is the program's thesis; this change accelerates the
  evaluation of unchanged DEC operators, it does not replace them.
- Cut cells, graded metrics, AMR (Stage 4+; `variable-grid-geometry.md`).
- Geometric multigrid (Jacobi-preconditioned CG suffices for what the
  direct solve does not cover at Stage 3 scales; multigrid is the
  documented escalation if cavity-scale CG iteration counts say so).
- Turbulence closures, compressible flow (Stage 5).
- DCT-IV / DST variants (not needed by the Neumann path; add on demand).

## Decisions

### D1: Stencil tables are a compiled evaluation strategy, owned by the solver

A new topology module compiles, per manifold, flat stencil tables for the
grade-0/1/2 operator pipeline the rate uses: for each output cell, the
input-cell indices (arithmetically enumerated once, in the canonical
cell-index order) and the fused coefficients (incidence sign × diagonal
Hodge factors × spacing weights). `d`/`δ` stop going through CSR on the hot
path — a cubical-lattice stencil row is 2·D entries at known offsets, and
loading CSR column indices from memory is exactly the traffic the tables
eliminate. Interior-product transport and sharp get the same treatment
(their gather patterns are manifold-constant; only the field values vary).

Ownership: the tables live in a `DecStencilTables<R>` value constructed
explicitly and held by the solver (`DecNsRate`/`DecNsSolver`), not hidden
inside `Manifold` — same conclusion as the FFT plan-cache decision (D8 of
`add-fft`): no struct surgery on `Manifold`, no type-erased caches.
Construction cost is one pass over the lattice (comparable to one rate
evaluation) and amortizes over the entire run.

Alternative considered — accelerating the generic CSR path (better CSR
layout, blocked matvecs): rejected; it keeps the column-index traffic and
the per-operator intermediates, which are the measured costs.

### D2: The rate streams through a fused workspace

`DecNsRate` gains a `RateWorkspace<R>` (scratch buffers sized at
construction). One stage evaluation = a fixed sequence of streaming passes
over the workspace with no allocation and no intermediate `CausalTensor`;
the convective and viscous terms share gathered neighbor values where the
stencils overlap. The public rate API is unchanged; the workspace is an
internal of the solver path. The generic compositional path
(`exterior_derivative` ∘ `interior_product` ∘ `laplacian`) remains intact
and is the equivalence oracle in CI.

### D3: Equivalence gates, not trust

For every operator and for the fused rate: stencil result vs generic
composition on randomized fields, ≤ 100·ε of the scalar (f64 ≈ 1e-13
absolute on O(1) fields), at f64 and Float106, on 2D and 3D lattices,
periodic and (once walls land) mixed. The existing Taylor–Green
convergence table must reproduce identically (same observed orders, same
table values at tolerance). A benchmark gate records the ≥ 2× serial rate
speedup; if the gate fails, the change does not ship a default switch to
stencils.

### D4: Spectral diffusion is per-stage, opt-in, and validated by the ladder

On fully periodic lattices, `Δ₁` block-diagonalizes per edge family (each
family is a shifted torus sub-lattice). The viscous term can therefore be
evaluated by three rFFT round trips with per-family eigenvalue multiply.
Decisions: (a) drop-in per-stage evaluation — RK4 semantics untouched;
(b) opt-in at solver construction (`with_spectral_diffusion()`), default
off until the validation ladder shows indistinguishable convergence
tables; (c) the eigenvalue tables reuse the `add-fft` per-axis-weight
machinery. Rationale for opt-in: unlike the Poisson solve (exact vs
tolerance-converged — strictly better), spectral diffusion changes the
*discrete operator's* rounding profile inside an explicit march, so it
earns default-on only through the ladder.

### D5: DCT enters `deep_causality_fft` as plan-based real transforms

`DctPlan<R>` with types I, II, and III (III = inverse of II up to the
crate's normalization convention), implemented on the existing power-of-two
core via the standard even-symmetric embeddings (DCT-II of length N via the
length-N rFFT of the reordered sequence with a twiddle post-pass; DCT-I via
the length-2(N−1) embedding), Bluestein-backed for awkward lengths exactly
like the complex planner. Same contracts as the rest of the crate:
immutable plans, caller scratch, allocation-free execution, naïve-DCT
reference for tests, normalization pinned in docs and round-trip tested.

Which type the Neumann solve uses is decided by the implemented operator,
not assumed: the path-graph Laplacian with half-cell boundary weights
diagonalizes in a cosine basis whose phase depends on the clipped Hodge
weights. The residual-at-rounding gate (same discipline as the torus
spectral solve) picks DCT-I vs DCT-II per the boundary-corrected `Δ₀` that
actually ships; both types are available.

### D6: Neumann dispatch extends `solve_laplacian`; mixed axes ride a complex carrier

The `add-fft` dispatch in `solve_laplacian` generalizes: grade 0 + uniform
Euclidean lattice + every axis either periodic or walled → direct spectral
solve with per-axis transforms (DFT on periodic axes, DCT on wall axes);
otherwise CG. For mixed-axis domains (Poiseuille: periodic-x, wall-y) the
field rides a complex carrier: complex FFT along periodic axes, DCT applied
independently to the real and imaginary parts along wall axes (the DCT is
real-linear), pointwise divide by `λ = Σ_d λ_d(k_d)` with the zero/Neumann
gauge mode zeroed, inverse. Costs 2× the memory of the pure-rFFT torus
path; accepted — correctness and one code path for all axis mixtures beat
a per-mixture specialization zoo.

The CG fallback gains a Jacobi preconditioner (diagonal of the
boundary-corrected `Δ₀`), shipped in `deep_causality_sparse` as an additive
`cg_solve_preconditioned`; it is the path for per-edge metrics and
non-uniform geometry, and the first preconditioner the roadmap deferred.

### D7: Boundary-corrected Hodge star via clip exponents

For a cell on open axes, the dual volume is clipped by `2^{-b}` where `b`
counts the open-axis boundary incidences of the cell (face → 1, edge → 2,
3D corner → 3). Implementation: the existing `CubicalReggeGeometry`
diagonal-star computation gains a clip-factor pass driven by the lattice's
periodicity flags and cell positions — no new types, the unit/uniform/
per-axis tiers all flow through it. The per-edge tier composes its own
corner products today and gets the clip factors in the same pass. The star
stays diagonal and positive, so CG's SPD requirement is preserved; a
dedicated test pins M-symmetry of the resulting `Δ₀`/`Δ₁` on walled
lattices.

### D8: No-slip as constrained tangential edges plus mirror-consistent rows

No-slip = zero tangential velocity at the wall. In DEC terms: 1-form
coefficients on wall-tangential edges are constrained to zero, and the
viscous operator's rows for interior edges adjacent to the wall use the
mirror (ghost) condition expressed without ghost storage — the wall-edge
contribution enters with the reflection sign folded into the stencil
coefficient. The constraint is enforced as a typed chain stage
(`bind: apply_no_slip`) after each projected rate application, and the
operator rows keep the modified stencil so the implicit-in-space
consistency holds. Symmetry of the constrained operator in the
M-inner-product is test-pinned (CG and the energy argument both need it).

Normal flux at walls is the projection's job (D6's Neumann condition), not
an edge constraint: wall-normal edges crossing the boundary do not exist on
an open lattice — the complex already trims them.

### D9: Solver wiring is additive

`DecNsSolver::new` accepts manifolds whose lattices are mixed-periodicity;
construction validates that every wall axis carries the boundary-corrected
star (the metric reports it) and wires: no-slip stage in the march chain,
wall-aware CFL (min spacing unchanged — uniform grids — but the advective
bound samples wall-adjacent speeds), seeding that respects the constraints
(seed fields are projected and constrained before the first step). The
periodic path is bit-unchanged when no wall axis is present.

### D10: Validation order is analytic-first, and the cavity reuses the entry harness

1. **Poiseuille** (CI): periodic-x, wall-y, body-force-driven; exact
   parabolic steady state. Gates: steady-state profile error converging at
   observed order ≥ 1.9; divergence-free at solve tolerance; no-slip exact
   at the wall by construction. This validates D5–D8 without corners.
2. **Lid-driven cavity Re 1000**: coarse rung (64²) in CI against the Ghia
   centerline tables with a generous-but-pinned RMSE gate and the
   convergence *trend* asserted; the full 129² run ships as an example
   program with the vortex-center table (primary + corner eddies) emitted
   — deliberately the same artifacts the CFD-challenge entry produces, so
   the comparison tooling is written once.
3. The Re-1600 64³ dissipation-curve example gains the stencil + spectral-
   diffusion configuration as its default once gates pass (Stage 1 exit
   artifact, now affordable).

### D11: Two tracks, one change, perf lands first

The tracks are independent until the benchmark task: the perf track
(D1–D4) touches only the periodic path and ships its gates standalone; the
walls track (D5–D10) builds on the unchanged generic operators and adopts
the stencil pipeline for wall lattices only at its end (stencil tables
carry the clipped coefficients transparently — they are compiled *from*
the corrected star). Tasks are grouped accordingly so partial landing
leaves the tree green and useful.

## Risks / Trade-offs

- [Stencil pipeline silently diverges from the generic operators as either
  evolves] → The equivalence tests are CI-permanent, not one-shot; any
  operator change that breaks them fails the build. The generic path is
  never deleted.
- [Boundary-corrected star changes existing open-lattice results] →
  Intentional and disclosed: previous open-lattice star values were
  interior-only. Affected tests are re-derived against the corrected
  mathematics (MMS where possible), never tolerance-loosened. The
  mixed-periodicity CG tests from `add-fft` are unaffected (unit metric,
  interior assertions).
- [DCT type / boundary-weight mismatch produces a subtly wrong direct
  solve] → The residual-at-rounding gate against the *implemented* `Δ₀` is
  the hard acceptance, exactly as it was for the torus; the preconditioned
  CG path cross-checks on every validation case.
- [The no-slip rows break operator symmetry and CG stagnates] → Dedicated
  M-symmetry test before any flow runs; the mirror-fold construction is
  chosen over row-zeroing precisely because it preserves symmetry.
- [Spectral diffusion's rounding profile shifts the convergence tables] →
  Opt-in until the ladder shows identical observed orders; remains opt-in
  if it does not.
- [Scope: G5 alone is ≈ G1+G2+G3] → The folded change is large by intent
  (user decision); D11's track structure keeps it landable in stages with
  the tree green between groups. If the walls track slips, the perf track
  still ships whole.
- [2× memory on the mixed-axis complex carrier] → Bounded and transient
  (one solve buffer); the all-walls cavity case uses the pure-real DCT
  path.

## Migration Plan

1. Perf track: stencil module + workspace + gates land behind the existing
   public APIs (no caller changes); benchmark records before/after; the
   solver switches to the stencil path once gates pass.
2. FFT crate: DCT plans land as pure addition (minor version bump).
3. Walls track: star correction → no-slip rows → Neumann dispatch →
   Poiseuille (CI) → solver wiring → cavity. Each lands green; nothing
   periodic regresses (the dispatch conditions are disjoint).
4. Rollback story: every piece is either additive or gated by dispatch
   conditions that can be narrowed; the generic operator path and plain CG
   are never removed.

## Open Questions

- DCT-I vs DCT-II for the implemented boundary-corrected `Δ₀` — resolved
  empirically by the residual gate during implementation (both ship).
- Whether spectral diffusion earns default-on — resolved by the validation
  ladder, not by this design.
- Jacobi vs symmetric Gauss–Seidel as the shipped preconditioner — start
  Jacobi (trivially SPD-safe, parallel-friendly); escalate only on
  measured cavity iteration counts.
