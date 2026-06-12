# Variable-Grid Geometry: The Under-Tackled Half of Vision 2030 Area 4

Status: exploratory note, 2026-06-10. Not a proposal; a starting point for deriving
one. Companion to `cfd-gap.md` (ground truth), `causal_cfd.md` (platform vision),
`3DCausalFluidDynamics.md` (analysis pipeline), `cfd-roadmap.md` (sequencing).

## 0. Provenance and motivation

NASA's CFD Vision 2030 study (NASA/CR-2014-218178,
https://ntrs.nasa.gov/citations/20140003093) states as Finding 4: "mesh generation
and adaptivity continue to be significant bottlenecks in the CFD workflow," and
dedicates investment Area 4 to making meshing "less burdensome and, ultimately,
invisible to the CFD process," explicitly calling for "automated optimal adaptive
meshing techniques" and "*in situ* anisotropic adaptive methods for time-dependent
problems."

A practitioner source: Dr. Tom-Robin Teschner (Cranfield PhD, Lord Kings Norton
medal; consulting clients include ITP Aero/Rolls-Royce, DSTL, Aston Martin,
McLaren), in "The 6 Biggest and Unsolved Challenges in CFD"
(https://cfd.university/blog/the-6-biggest-and-unoslved-challenges-in-cfd):
geometry and grid generation is severely underfunded relative to its importance —
research papers on turbulence outnumber grid-generation papers by a factor of
**42** — and real simulations essentially never run on the uniform mesh that solver
papers quietly assume. He additionally flags that "no good format to exchange CAD
data" exists: CFD needs watertight, singularity-free geometry that most CAD formats
do not deliver (relevant to the cut-cell phase, `causal_cfd.md` §4.2/§4.9). The
strategic inference: the thinner field with the harder problem is where new value
is created, and NASA put geometry/grid into the report for exactly this reason.

This note examines why the problem is thorny, why the DeepCausality stack holds a
structural advantage on it that incumbents cannot retrofit, and what a staged
attack looks like.

## 1. The problem, and why it is thorny

Resolution must follow the physics. A turbulent boundary layer at flight Reynolds
numbers needs wall-normal spacing three to four orders of magnitude finer than the
outer flow tolerates; shear layers, wakes, and shocks impose the same demand along
different directions. A uniform mesh fine enough for the wall is computationally
absurd everywhere else; a mesh coarse enough for the freestream resolves nothing
that matters. Every production simulation is therefore graded, stretched,
anisotropic, locally refined, or all four at once.

Why the field underinvests despite this: the difficulty is *coupled*, not isolated.

1. **Mesh quality couples invisibly to solver behavior.** Skewness, aspect ratio,
   and growth-rate jumps degrade stability and accuracy through mechanisms the
   solver does not report; bad meshes kill solutions silently (`causal_cfd.md` §1).
2. **Refinement interfaces break conservation.** Hanging nodes and level jumps in
   AMR require carefully constructed transition operators or fluxes leak.
3. **Moving/adapting meshes break the geometric conservation law.** A scheme that
   conserves on a static mesh can spuriously generate mass on a deforming one.
4. **Anisotropy breaks most error estimators**, which assume locally isotropic
   resolution.
5. **Parallel adaptivity data structures are hard**, and they entangle the mesh
   with the solver's memory layout.

Conventional codes store vertex coordinates and bake the metric into every stencil:
connectivity and geometry are fused in the data structure, so every one of the five
couplings above touches everything.

## 2. The structural asset: Regge's separation of topology and geometry

The DeepCausality stack inherits, from its general-relativity lineage, the one
architectural decision this problem rewards: **topology and geometry are separate
objects.**

- `LatticeComplex<D, R>` is pure combinatorics — cells, boundary operator,
  orientation. It knows nothing about lengths.
- `CubicalReggeGeometry<D, R, S>` carries the geometry as **per-edge lengths** — a
  *field* over the fixed lattice, exactly as Regge calculus represents curved
  spacetime on a fixed triangulation. The Hodge star consumes these lengths
  (verified in the `cfd-gap.md` audit: `has_hodge_star.rs`, `volumes.rs`).
- The DEC operators split along the same line: `d` is metric-free and purely
  combinatorial; **all** geometry enters through `⋆`.

Two consequences, and they are the whole thesis of this note:

**A variable mesh is a metric state, not a data structure.** Grading, stretching,
and anisotropy are edge-length assignments on an *unchanged* lattice. No remeshing,
no connectivity surgery, no coordinate arrays — the "mesh generation" step for a
graded grid is writing a function `edge → length`.

**Conservation is combinatorial, so grading cannot break it.** `d∘d = 0` and the
discrete Stokes theorem hold for *any* edge-length assignment, because they never
see the metric. The divergence-free-by-construction property of the Leray projector
(`cfd-gap.md` §2) is therefore **grading-independent**. On a stretched, anisotropic,
arbitrarily ugly metric, only *accuracy order* is at stake — never structure. In a
conventional code, both are at stake on every bad cell. This is couplings 1–3 of §1
partially dissolved by architecture rather than managed by craftsmanship, and it is
not retrofittable to a code that stores xyz per node and bakes the metric into its
stencils.

One more inherited asset: `cubical_regge_geometry/metropolis.rs` already implements
stochastic *evolution of the edge-length field* (built for Regge quantum-gravity
updates). Mesh adaptation as metric dynamics has existing code paths.

## 3. Three rungs

### R1 — Graded lattices (metric variation only; near-term)

Variable `dx` per edge with zero new data structures.

- Graded constructors on `CubicalReggeGeometry`: geometric and tanh stretching laws
  (the standard wall-normal grading families), per-axis.
- Verify the operator stack end-to-end on graded metrics: the Hodge star path
  already consumes per-edge lengths; G1's wedge/interior-product property tests
  (Leibniz, Cartan) must run on graded metrics, not only uniform ones.
- Measure, honestly: MMS (graded Taylor–Green) truncation study. Smooth grading
  should retain second order; abrupt growth-rate jumps degrade locally — quantify
  the growth-rate limits. Conservation and divergence-freeness are asserted exact
  (machine/CG tolerance) at *every* grading, per §2 — that assertion is the
  headline test.

**Direct consumer: `cfd-roadmap.md` Stage 3.** Poiseuille and lid-driven cavity at
Re 10⁴ are not credible on uniform meshes; wall-normal grading is how every real
channel/cavity computation resolves the boundary layer. R1 should therefore land
*with or just before* Stage 3. Effort: small — constructors plus a test battery;
the operators already do the work.

### R2 — Anisotropic metric adaptation (medium-term)

The metric-based mesh-adaptation literature (the continuous-mesh framework of
Loseille & Alauzet) treats the ideal mesh as a Riemannian metric field and then
struggles to realize it in coordinate meshes. Here the metric field **is the native
data structure** — adaptation is evolving the edge-length field from an indicator,
with no mesh realization step at all.

- Indicators available today: gradient, vorticity, Q-criterion (shipped kernels).
- Indicator absent: adjoint-based goal-oriented adaptation (the field's gold
  standard; the platform has no adjoints — same gap as `causal_cfd.md` §0.2 table,
  area 6).
- **The novel indicator: causal adaptation.** Stage 2's attribution pipeline
  (`3DCausalFluidDynamics.md`) identifies which regions' state *causally drives* a
  quantity of interest. Adapting where causal influence is high — rather than where
  gradients are large — is goal-oriented adaptation **without adjoints**, using an
  information stream no other code possesses. Speculative, publishable, and the
  second place (after closure discovery) where the causal program and the numerics
  program intersect rather than merely coexist.

### R3 — Topological refinement (true AMR; the research rung)

Metric grading redistributes resolution but conserves cell count; it cannot *add*
degrees of freedom, and extreme stretching eventually degrades the dual-mesh
quality inside the Hodge star. Strongly localized features (shocks, contact
surfaces) need subdivision: 2:1-balanced octree refinement on the lattice.

The hard part is honest to name: a non-conforming refinement interface breaks the
clean chain-complex structure that §2's guarantees ride on. Candidate routes —
conforming transition templates on cubes, mortar/chain-map coupling between
refinement levels, or a genuinely non-conforming DEC. The literature on
DEC-with-hanging-nodes is thin, which per §0's fifty-to-one argument is precisely
the opportunity: whoever makes refinement interfaces *provably* conservative inside
a chain complex has solved the coupling that every AMR code manages by hand. R3 is
research, not wiring, and replaces the "provisional AMR" line in `causal_cfd.md`
Phase 4 with a concrete mathematical program.

## 4. Limits of the metric-only approach (stated up front)

- Grading conserves DOF count; it cannot refine *and* keep the far field — R3
  exists for a reason.
- Extreme anisotropy stresses the primal–dual volume ratios in the star; the R1
  truncation study must report where accuracy collapses, not only where it holds.
- Cut cells (`causal_cfd.md` §4.2) interact with grading at walls; the R1
  constructors should anticipate the cut-cell phase by keeping the wall-normal
  axis-aligned case first-class.

## 5. Roadmap hooks

| Rung | Lands with | Consumes | Unblocks |
|---|---|---|---|
| R1 graded metrics | Stage 3 (walls) — credibility prerequisite for Re 10⁴ cavity | G1 operator tests on graded metrics | Boundary-layer-resolved validation throughout |
| R2 metric adaptation | Stage 4 companion | R1 + Stage 2 attribution (causal indicator) | Goal-oriented adaptation without adjoints |
| R3 topological AMR | Stage 5 / replaces provisional Phase 4 AMR | R2 limits study | Shock-class features; Vision 2030 "in-situ anisotropic adaptive methods" |

## 6. Why this is the right hard problem

Vision 2030 Area 4 is the report's most operationally painful finding and its least
crowded research field (≈50:1 against, per the practitioner observation). The
incumbents' meshing pain is simultaneously their moat (services revenue) and their
wound (the workflow bottleneck their architectures cannot dissolve). The
topology/geometry separation is a property the platform already has — built for
general relativity, free for CFD — and it converts the field's hardest *systemic*
coupling (mesh ↔ conservation ↔ stability) into a theorem instead of a craft. That
is the same pattern as the rest of the platform story: the differentiating property
was not added for CFD; it was already there.

## 7. Open questions for the proposal that derives from this note

1. R1 grading-law API: per-axis analytic laws vs. arbitrary per-edge assignment
   (probably both; the analytic laws as constructors over the general field).
2. Where the R1 truncation study's growth-rate limits sit relative to standard
   practice (industry rules of thumb: growth ratio ≤ 1.2–1.3 near walls).
3. Whether the causal indicator (R2) needs the full attribution pipeline or a
   cheaper single-snapshot surrogate (per-region SURD on the decomposition the
   solver already emits).
4. R3 route selection: transition templates vs. mortar chain maps vs. non-conforming
   DEC — a literature pass plus a 1D/2D toy before committing.
5. Interaction with `MaybeUncertain` zones: uncertain *geometry* (moving surfaces,
   `causal_cfd.md` §3.1 item 5) as uncertain *edge lengths* — `MaybeUncertain<R>`
   per edge is representable today; is it useful?
6. Overset/moving-frame meshes: Vision 2030 Area 4 explicitly asks for "highly
   scalable dynamic structured and unstructured overset mesh technology," and the
   practitioner work this note leans on uses moving reference frames + overset
   meshes routinely (references.md: Rijns-2024-corner). Overset is R3-adjacent
   territory this note does not cover — does the topology/geometry separation
   offer anything for inter-grid interpolation (chain maps between overlapping
   complexes), or is overset simply out of scope for a lattice-based code? A
   literature-pass question, not a commitment.
