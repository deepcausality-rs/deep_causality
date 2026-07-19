# CFD roadmap: near-term wins against the field's open challenges

> **STATUS: exploratory note (2026-07-03).** Nothing here is proposed or built. This note mines
> CFD University's survey of the six biggest unsolved challenges in CFD
> ([source](https://cfd.university/blog/the-6-biggest-and-unoslved-challenges-in-cfd)) for the
> sub-problems `deep_causality_cfd` could answer with minor updates or additions, and ranks
> them from least to most effort. The crate README records what is answered *today*; this note
> records what one small step would add.

## The six challenges, summarized

What the survey actually says, so the rest of this note has a fixed reference:

1. **High-performance computing.** Legacy CFD codes scale to thousands of cores; exascale
   needs a million, on heterogeneous CPU-GPU hardware, and the output volumes outgrow human
   pre- and post-processing entirely.
2. **Turbulence modelling.** RANS models assume fully turbulent flow and miss
   laminar-to-turbulent transition; transition correlations do not generalize past their
   calibration cases; scale-resolving methods (LES) must capture roughly 80 percent of the
   turbulent kinetic energy, and solvers give no standardized support for verifying that.
3. **Numerical algorithms.** Convergence at high Reynolds number is judged by expert feel, not
   objective measures; uncertainty quantification is underdeveloped; the errors from
   turbulence models, boundary conditions, and numerics cannot be isolated from one another;
   linearized error estimators grow without bound on time-dependent flows.
4. **Geometry and grid generation.** The most underfunded bottleneck (the survey counts 42
   times more papers on turbulence than on grids): no automatic high-quality meshing for
   complex geometry, and no CAD format that is watertight and singularity-free by contract.
5. **Knowledge extraction.** Post-processing remains manual while output grows exponentially;
   higher-order solutions cannot even be visualized natively; reduced-order models and
   simulation-experiment data fusion stay underused.
6. **Multidisciplinary analysis and optimization (MDAO).** Coupling CFD to structural,
   thermal, and flight-dynamics solvers has no standardized framework; the propagation of
   uncertainties from solver to solver, tracking overall simulation uncertainty, is simply
   absent; no CGNS-equivalent data format exists for multidisciplinary results.

## The scorecard, restated

The crate already answers parts of three (MDAO coupling, error attribution, knowledge
extraction; see the crate README). Several remaining sub-problems sit one small addition away;
they make up the ranked list below. Turbulence modelling is **in scope but staged**: it comes
after nearer roadmap gaps close, and it has a genuine dependency on one of them (see the staged
section after the list). Only million-core scaling stays out of scope at any effort tier.

## Ranked list, least to most effort

### 1. Objective convergence acceptance, as the default workflow

**Challenge 3 (numerical algorithms).** The survey's complaint: judging convergence at high
Reynolds number takes domain expertise because no objective measure is standard. The crate
already has the machinery: MMS builders, operator-accuracy studies, and gates on observed
convergence order, all exiting nonzero on regression. What is missing is nothing but
replication of the existing pattern onto each new solver configuration as it lands.
**Effort: near zero.** No new machinery; a house convention, already practiced, stated as a
rule.

### 2. More tensor-train-native observables

**Challenges 1 and 5 (automated post-processing, knowledge extraction).** The survey argues
exascale output volumes make manual post-processing impossible. The crate's answer is to
extract observables in the compressed representation (`drag_lift`, `wall_heat_flux`,
`divergence_residual`, `max_bond`) and gate them, so no human reads fields. Extending the
answer means new extractors: vorticity and circulation, kinetic-energy spectra, surface
distributions. Each is a small pure function beside the existing ones, with tests mirroring the
current set. **Effort: small.** One function plus tests per observable; the codec and gating
infrastructure already exist.

### 3. Dispersion sweeps as a first-class flow combinator, carried by `Uncertain<T>`

**Challenge 6 (MDAO uncertainty propagation).** The survey calls solver-to-solver uncertainty
propagation "simply absent" from current practice. The weather-dispersion example already does
it end to end: six counterfactual atmospheres times eight deterministic noise draws, pushed
through the whole coupled chain, reported with error bars, significance gated. But it lives as
example code, and its statistics are hand-rolled. This is where `deep_causality_uncertain`
changes the shape of the answer rather than just its packaging. `Uncertain<T>` is a first-order
uncertain type (after Bornholt et al.): a declared dispersion becomes a typed distribution
(`normal`, `uniform`, `point`) instead of a hand-written world list, operations build an
implicit computation graph that propagates the uncertainty lazily, and decisions run as
sequential hypothesis tests (SPRT) that draw only as many samples as the decision needs at a
stated confidence. Three consequences for the sweep combinator. First, the weather table's
hand-rolled significance gate (mean, sigma, a two-sigma rule) becomes
`probability_exceeds(...)` with the confidence explicit. Second, the flat 48-descent cost
becomes adaptive: an obvious effect resolves in few draws, a marginal one buys more
automatically. Third, the plumbing already exists, because the CFD crate depends on
`deep_causality_uncertain` today and ships the sensor-fed `UncertainMarchPipeline` that
consumes `Uncertain` inflow; deterministic seeding (`seed_sampler`) and a QMC sampler keep the
house bit-reproducibility rule intact. The addition is a `flow_config` builder that takes a
baseline description, dispersions declared as `Uncertain<T>` values, and a confidence target,
and returns per-condition statistics as one owned report. **Effort: small to moderate.** The
uncertain type, the samplers, and the proven example all exist; the work is the typed surface
that joins them.

### 4. Reduced-order model export from QTT states

**Challenge 5 (knowledge extraction).** The survey notes that building reduced-order models
from high-fidelity CFD is underused despite large payoffs. A quantized tensor train *is* a
reduced-order model with a tunable error knob. The addition is an API that rounds stored
solution states to a target tolerance or bond cap, re-evaluates the registered observables on
the rounded states, and reports the observable error alongside the compression ratio. The
pieces (`Truncation`, `quantize`/`dequantize`, `max_bond`, the observable extractors) all
exist, and the field-tier snapshot container from `add-cfd-file-io` (shipped 2026-07-03) is
the export's payload format. **Effort: moderate.** New API surface and a verification example,
but no new numerics.

### 5. Self-describing multidisciplinary results

**Challenge 6 (MDAO data format).** The survey points out that no CGNS-equivalent exists for
multidisciplinary results, which makes cross-domain collaboration inefficient. The crate cannot
declare a community standard, but it can ship a self-describing artifact: one serialized
archive holding the world description (the owned config), the `Report` series, and the
provenance `EffectLog`, so a result names its own inputs and audit trail. CSV output and the
`IoAction` seam exist; the work is schema design and round-trip tests, and two payload pieces
shipped with `add-cfd-file-io` (2026-07-03): the units-aware result table and the checksummed,
fingerprinted snapshot container the archive can adopt unchanged. **Effort: moderate.**
Design-heavy rather than code-heavy, and worth a short spec before building.

**Elaboration: how a serialized config relates to the missing CGNS-equivalent.** CGNS
standardizes the answer (fields on a grid); a serialized CfdFlow configuration standardizes
the question (world, solver, coupling, schedule, gates). For multidisciplinary results the
question half is the load-bearing half: a single-discipline field is self-interpreting, but a
coupled result ("polar-winter drift 68.75 m") carries no meaning without the IMU model, the
atmosphere, the coupling order, and the closures that produced it, and that description layer
is what the community never standardized. Because configuration and execution are separated in
this crate, the serialized description also *executes*: same config plus deterministic
execution reproduces the same bits, so the format is a reproducibility contract rather than
metadata that can rot. The full CGNS-equivalent for this ecosystem is then the triple this
item already names: description (config), result (`Report`), provenance (`EffectLog`), with
two properties CGNS never had (executability, and acceptance criteria that travel with the
result) and one it cannot claim (multi-implementation adoption; the format is executable only
by this engine, though third parties can read and validate it from an open schema).

**Elaboration: the enterprise process model this enables.** With an open config schema, third
parties generate configs for a catalog of standardized pipelines: import a watertight CAD
surface, emit a config for "process 42", load it, run it. The design splits along the line
that model needs. The *process definition* stays a versioned, typed Rust artifact with its
gates compiled in (a qualified process should be frozen; it cannot be mis-assembled from a
file), while the *parameter surface* is open data the config selects and fills
(`process = "plasma-blackout-corridor@1"` plus values). The weather table already runs this
shape today: one process, six parameter sets, 48 runs, one gated table. Every third-party
parameterization self-verifies against the process's own pinned bands, with the exit code and
provenance log as the QA record. Additions this implies beyond the archive itself: a process
registry (named, versioned flow definitions addressable from a config), schema-validated
untrusted input (the `DescentSchedule::new` validation is the seed of that posture, applied to
every config field), a thin runner (load config, run, emit the archive, exit code from the
gates), and the item-7 rasterizer as the CAD entry point. This is the simulation-process-and-
data-management (SPDM) layer enterprises currently build as scripting glue around closed
solvers, expressed as typed artifacts instead.

### 6. Experimental data fusion into states and gates

**Challenge 5 (knowledge extraction, data fusion).** The survey flags fusing simulation with
experimental data as underdeveloped. The crate holds both halves already: the sensor-fed
uncertain-inflow march consumes measured boundary data with quantified noise, and the 17-state
error-state Kalman engine folds measurements into an evolving state. The addition is an
assimilation stage that folds measured *flow* observables (a pressure tap, a heat-flux gauge, a
reflectometer trace) into the coupled state with a per-observable measurement model, and a gate
form that scores prediction against measurement with the noise budget explicit.
`deep_causality_uncertain` helps here too: `MaybeUncertain<T>` models a value whose *presence*
is uncertain, which is exactly what an intermittently reporting gauge or a dropout-prone
telemetry channel is, so sparse experimental data gets a type instead of a sentinel value.
**Effort: moderate to large.** The estimation machinery exists; the per-observable measurement
models need real design, and each fused quantity needs its own validation story.

### 7. Meshless complex geometry through watertight-surface rasterization

**Challenge 4 (geometry and grid generation).** The survey calls grid generation the field's
most underfunded bottleneck and asks for watertight, singularity-free geometry handling. The
crate's immersed-boundary path (`mask_from_fn`, `body_mask_2d`, the immersed QTT solver)
already avoids meshing for analytic shapes. The addition is a rasterizer from a watertight
closed surface (an STL or an implicit function) to a signed-distance mask on the uniform QTT
grid, which would extend "no meshing" from analytic bodies to arbitrary closed geometry.
**Effort: largest on this list.** A 3-D mask path, geometry robustness work, and a fresh
verification ladder (a benchmark body with published references); still bounded, because it
bypasses mesh generation rather than solving it. The section below elaborates what the QTT
representation changes about this challenge as a whole.

## What QTT changes about geometry and grid generation (challenge 4, elaborated)

The QTT representation contributes three structural things to the meshing problem, bounded by
two measured limits.

**The grid is never generated.** A QTT field lives on a uniform `2^L` grid addressed by the
binary digits of the coordinates. There is no meshing step: no cell-quality metrics, no
skewness bounds, no smoothing pass, no human in the loop. Refinement is `L -> L+1`, one more
tensor core, and the cost grows as `chi^2 * L` rather than by a factor of eight per octave.
The automation sub-problem the survey names has nothing left to automate, because nothing is
generated.

**Compression removes the economic reason graded meshes exist.** Meshes are graded (boundary
layer stacks, wake refinement, shock adaption) because a dense uniform fine grid is
unaffordable, so a human decides where the resolution budget goes. QTT charges by field
structure instead of point count: a smooth region costs rank near 1 no matter how many grid
points cover it, so global fineness is bought at logarithmic storage cost and structure is paid
for only in bond dimension, where the field actually has it. The trade-off that mesh grading
negotiates is the thing the representation deletes.

**Geometry becomes data with a minimal CAD contract.** The immersed path (`mask_from_fn`,
`body_mask_2d`, the immersed QTT solver) rasterizes the body as an indicator or signed-distance
field on the uniform grid, and that mask is itself a QTT with the same codec and rounding as
the flow fields. The only property this asks of the upstream CAD is that inside/outside is
well defined, which is watertightness, exactly the guarantee the survey asks CAD formats to
provide. Surface meshing, boundary-layer extrusion, and mesh-ready topology drop out of the
contract entirely. Item 7 above (the watertight-surface rasterizer) is the bridge that extends
this from analytic shapes to arbitrary closed geometry.

**Measured limit one: alignment sets rank.** The crate's own rank studies are the fence. A
captured curved shock on a misaligned Cartesian grid grows rank without bound (`qtt_rank_3d`
measured chi ~ sqrt(side), 45 to 135 over 16³ to 128³), while shock-aligned and body-fitted
coordinates hold chi ~ 6 constant. Discontinuities therefore still need shock fitting (the
compressible carrier's answer: the Rankine-Hugoniot state as the boundary of the marched
layer) or an analytic coordinate map (`src/coordinate/`). A map is a handful of analytic
functions, closer to choosing a conformal transformation than to generating a mesh, but for a
complex assembly a good global map may not exist; that remainder of challenge 4 stays out of
scope, as recorded below.

**Measured limit two: the wall, not the grid.** Storage says a `2^L` grid can resolve a
boundary layer at log cost (a laminar profile is smooth, hence low rank); the working
constraints sit in the solver, not the representation: near-wall accuracy at an immersed
boundary needs the cut-cell no-slip treatment (the proposed `add-aperture-resolved-noslip`
design), and fine grids tighten the explicit time step, which is what the acoustic IMEX and
preconditioner studies address. High-Reynolds wall-bounded claims wait on both.

One cheap study would pin the open empirical question here: the rank of SDF masks as body
complexity grows (the geometric analog of the existing rank studies). Smooth boundaries should
stay low rank with structure concentrated near the surface; measuring where that stops holding
would size item 7's verification ladder before it is built. Net effect of the whole section:
QTT shrinks what must be generated from a volume mesh to an inside/outside query plus at most
one analytic coordinate map.

## Turbulence modelling (challenge 2)

Turbulence is in scope and will be added; it is sequenced after the ranked items above rather
than excluded, because it is a change series of its own and because one of the small items is
its literal prerequisite. The natural entry point is scale-resolving simulation on the DEC
solver, which already runs the laminar validation ladder (Taylor-Green, Couette and Poiseuille,
the Re-1000 cavity, cylinder wakes): add an explicit subgrid-scale closure and the LES path
inherits a solver whose incompressibility is exact by construction. The survey's sharpest
complaint in this area is not the closures themselves but that "solvers provide no standardized
support" for verifying the resolved-energy criterion (capturing roughly 80 percent of the
turbulent kinetic energy). That maps directly onto this crate's gate culture: a resolved-TKE
fraction computed per run and gated, so LES adequacy becomes an executable check instead of a
reviewer's judgment call. The dependency: measuring resolved TKE needs kinetic-energy spectra
as observables, which is item 2 on the ranked list. Sequencing therefore falls out on its own:
item 2 first, then the SGS closure with its verification ladder as a dedicated OpenSpec change
series.

## Adoption preconditions (release-gating, added 2026-07-03)

What must be true for a practitioner to use the toolbox, independent of language preference.
The release criterion, stated once: **a reasonable toolbox that works for a clearly named
category of problems, well documented, with an explicit line between what works today and
what is aspirational on this roadmap.** The everyday-examples note defines today's category
(the five desks); this section carries the adoption items not otherwise owned.

- **One-command start.** The crates are `publish = false` today, so a practitioner must clone
  the monorepo. Either publish the crate stack or ship a study-template repository whose
  `cargo run --release` works on the first try. Small; unblocks the study-as-script tier.
- **Dense-field export for visualization (VTK/Tecplot).** Practitioners judge a flow by
  looking at it before they believe a gate; `dequantize` already produces the dense field, so
  the export is a small writer in `deep_causality_file` beside the result table. Load-bearing
  for trust; small.
- **The config runner (the no-Rust tier).** A thin prebuilt binary that loads a serialized
  config for a registered process and runs it: item 5's config seam plus the process-registry
  and runner from the enterprise elaboration above. The largest adoption prize; moderate, and
  already scoped by item 5.
- **A stability promise.** Engineers re-run deliverables at audit time, sometimes years
  later: versioned processes, the snapshot format's version field (shipped), and a stated
  compatibility policy. Mostly writing, but it must be written before a release claims it.
- **Practitioner-grade errors** are owned by the everyday-examples note (Group 1, item 5):
  every reachable error speaks engineering and names the artifact involved.
- **Documentation as a cookbook, plus a validation-and-verification page** (one table of all
  deviations from reference results, per solver, on the project website): noted and owned by
  the user, to be specified at a later stage; recorded here only so the release checklist is
  complete in one place.

## Out of scope at any effort tier

Named for honesty, so this note is not mistaken for a claim of coverage: million-core and
CPU-GPU scaling (challenge 1), and CAD repair or automatic body-fitted mesh generation for
complex assemblies (the rest of challenge 4). The crate's leverage lies where its architecture
already carries the weight: typed coupling, compression, determinism, and gates.

## Suggested sequencing

Items 1 through 3 are candidates for a single small change each; item 3 (the dispersion
combinator) has the highest value density because it turns the survey's sharpest MDAO gap into
a one-builder feature backed by an already-validated example. Item 2 doubles as the
prerequisite for the staged turbulence work, which raises its priority beyond its own size.
Items 4 and 5 pair naturally (a ROM export wants a self-describing container). Items 6 and 7
each deserve their own OpenSpec change with a design phase before any code, and turbulence
follows as its own change series once item 2 has landed.
