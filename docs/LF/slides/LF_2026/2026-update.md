# DeepCausality: Major Updates, January 1 to August 25, 2026

Scope: new crates, new capabilities, and project-level events. Minor fixes, test-coverage work,
and dependency bumps are excluded. Sources: merged pull requests, the commit history, the archived
OpenSpec change log, crates.io publication records, and the project blog.

---

## January 2026

**Gauge theory across topology and physics.** The topology crate gained a `GaugeField` type with
HKT instances, followed by a `LatticeGaugeField` (#448) and a certified version of it (#450). The
`SE(3)` gauge group of rigid-body motions arrived on January 20 (#457). On the physics side this
supported four gauge theories with worked examples: QED (renamed from `qed` to `electromagnetism`),
the weak and electroweak theories, and a gauge-based formulation of general relativity. ADM
operations became generic over `Field`.

**Float106, a double-double precision type.** Introduced on January 6 as `DoubleFloat` and renamed
to `Float106` on January 22 (#459). Roughly 106 bits of mantissa from a pair of `f64` values.
Topology and physics were made generic over it in the same month.

**`RealField` surface extensions:** `asin` with a default implementation (#456), and
`is_nan` / `is_infinite` / `is_finite` (#464).

## February 2026

## March 2026

No major feature work. Dependency, SBOM, and Bazel maintenance only.

## April 2026

No major feature work. Dependency and build-tooling maintenance only.

## May 2026

**Stateful Causaloid evaluation** (#506). A new API for Causaloids that carry state across
evaluations, shipped together with the `flight_envelope_monitor` avionics example.

**Spec-driven development.** The repository migrated to OpenSpec on May 3. Every substantial change
since then has a written specification, a task list, and an archive entry.

**The papers repository was merged into the monorepo** (#515, closing #508). Publications were later
reorganised so each paper sits in the crate that implements it.

**Chronometric physics kernel** (#523): J2-corrected weak-field GM inversion, with an example
recovering the gravitational parameter from clock data.

**A new project website** (#539 through #549). Built with Astro, deployed on Cloudflare Workers,
with the blog migrated and worked examples on the front page. Documentation followed on May 28 as a
standalone Starlight site at `docs.deepcausality.com` (#594), with 301 redirects from the old
in-site paths (#597).

**Miri in CI across the whole repository** (#552, #553, #555). Phased in over one day, from a single
pilot crate to all 20 safe crates.

**Cubical complexes** (#557). `LatticeComplex`, cubical aliases, neighbourhood strategies, and a
`ChainComplex` trait that `Manifold` was made generic over.

**Isomorphism traits** in `deep_causality_num` and `deep_causality_haft` (#560, #562). Tier-1 marker
subtraits and Tier-2 witness-typed `Iso` traits.

**The `RealField` generalisation sweep** (#568, #569, #570). Topology, physics, and the effects crate
lost their hard-coded `f64`. Roughly forty commits over two days moved unit types, kernels, and
solvers onto a generic real scalar.

**Cubical Regge calculus** (#571, #573): cell volumes, hinges and dihedrals, deficit and action, a
Lorentzian signature marker with a Wick-rotated action, an action gradient, and a
Metropolis-Hastings sampler. Alongside it, Hodge decomposition with a matrix-free conjugate gradient
and a `TopologicalInvariants` extractor.

**Fluid dynamics kernels in physics** (#580). Fourteen groups covering kinematics, viscous stress,
dimensionless numbers, turbulence quantities, coherent-structure detectors, compressible
thermodynamics, and boundary layers, plus four Navier-Stokes regime evaluators: Stokes, Euler,
incompressible Newtonian, and compressible Newtonian.

**Graph and geometry additions:** Delaunay triangulation for point clouds (#584) and
biconnected-components decomposition in ultragraph (#583).

**Blog.** Eight posts published between May 17 and May 31, including the website announcement, four
explainer posts on correlation, regime change, distribution shift, and LLM limits, a release summary
for 0.13.7, and "Counterfactuals via the Causal Monad".

## June 2026

**BRCD, a second causal-discovery algorithm** (#611). Bayesian root-cause discovery: an exact AMO
enumeration MEC engine over a new `MixedGraph` type, Meek orientation rules, a logistic-regression
gate by IRLS, a ridge-Gaussian family estimator, F-node augmentation with cut-config enumeration,
and posterior assembly. BOSS structure learning was added underneath it on June 3, with a bootstrap
CPDAG-uncertainty variant. BRCD was wired into the Causal Discovery Language on June 8 (#616), with
a learn-once, rank-many CPDAG cache. Later in the month it gained parallel evaluation across
candidates, opt-in near-linear MAP-config pruning, and a `dag_sampling` module with a
polynomial-time Clique-Picking AMO counter and a uniform MEC DAG sampler.

**The Causal Arrow generalization** (#613). The causal monad was generalised to an Arrow: `Morphism`
and `Endomorphism` in the algebra tower, a value-level Arrow algebra with a builder, an `Either`
choice sum in haft, forward-mode autodiff over `Dual`, and numeric integration operators (Euler,
RK4, quadrature). The `CausalFlow` fluent facade landed on June 5, giving the monad's power without
its type-level ceremony. `deep_causality_calculus` was first published to crates.io on June 8.

**A DEC-native Navier-Stokes solver.** Built in three steps: exterior algebra, de Rham transfer, and
Leray projection (June 11); the periodic incompressible solver (June 11); then wall-bounded flow with
constrained Leray projection, no-slip walls, and moving lids, validated against Poiseuille flow and
the Ghia cavity benchmark (June 12).

**Two new crates for the solver stack:** `deep_causality_fft` (complex, real, and N-dimensional
transforms, with a spectral Poisson solve) and `deep_causality_par` (shared parallelism primitives).
Both were published to crates.io on June 17.

**Cut cells and immersed boundaries** (June 14 and 15). A cut-cell geometry substrate, a cut-aware
Hodge star, small-cut-cell stabilisation by cell merging, composable boundary zones with static
dispatch, inflow/outflow and free-slip zones, a net-flux open-boundary Leray projection, pressure
surface-force diagnostics, and aperture-resolved cut-face no-slip through a weighted KKT projector.

**`deep_causality_cfd`, a new crate** (#629, #637). The fluid stack was moved out of physics into a
dedicated crate on June 15, together with the Flow DSL facade, immersed-body diagnostics
(drag and lift, wake probe and Strouhal number, centerline), zones, and a `.couple` seam for
multiphysics.

**Quasi-Monte-Carlo sampling.** Sobol sequences and inverse-CDF transforms in rand, a `QmcSampler`
in the uncertain crate, and an opt-in QMC collapse for the uncertain inflow (June 16). Deterministic
sampler seeding was added at the same time.

**The IO monad** (#629 group, June 17). A lazy `IoAction` effect in haft as the Arrow twin of the
existing effects, file IO actions in core with a `CausalFlow` read/write bridge, and CSV writers that
run through the effect rather than around it.

**A tensor-network stack in `deep_causality_tensor`** (June 26 to 28). Truncated SVD and QR,
`CausalTensorTrain` (MPS), `CausalTensorTrainOperator` (MPO), TT-cross, an ALS solve engine, DMRG3S,
two-site TDVP2, adaptive randomized TT-rounding, the full TT operator algebra, and a complex-capable
path throughout via a `ConjugateScalar` trait, including a complex Hermitian eigensolver.

**A QTT solver in CFD** (June 28 to 30). A quantized-tensor-train bridge (codec, finite-difference
operators, linear rollout), a 2-D incompressible Navier-Stokes solver, an immersed body with surface
observables, then the Tier-B compressible marcher: 3-D operators, a body-fitted coordinate, a
Sod-gated compressible flux, an IMEX split-acoustic integrator, a closed-form acoustic inverse, and
a `MetricProvider` seam that makes body fitting data rather than a code path. Park-2T hypersonic
reacting and ionization kernels were added in physics to support it.

**`deep_causality_file`, a new crate** (June 30). Data loading and processing over the haft IO monad,
including RINEX GNSS SP3/CLK receiver data.

**Examples:** a closed-loop corrective DDoS detector (#623), an integrated DeepCausality and Candle
ML example (#632), an ML root-cause-analysis example, and an INS/GNSS-blackout clock-holdover
example.

**Governance:** an official statement on the use of AI coding assistance was published on June 2
(#610).

**Blog:** two posts on June 9, "The Causal Discovery Language, Rebuilt" and "The Causal Flow
Language".

## July 2026

**The plasma-blackout corridor** (July 1 and 2). A Kustaanheimo-Stiefel conformal core with
constraint projection, a 3-D `MetricProvider3d` seam, a body-fitted spherical-shell metric, a 3-D
compressible marcher coupled to real aerodynamic forces, a navigation engine with a 17-state error-
state Kalman filter and regime switching, synthetic sensors, and a closed-loop navigation gate. The
corridor example chains all of it in the `CfdFlow` DSL, with a Park-2T controller, wall heat flux,
IMU drift, and noisy GNSS fixes.

**The CFD Flow DSL rework** (#665, July 3 to 5). A campaign grammar with study phases, gates, and a
verdict; a `StudyEffect` substrate following the CDL effect pattern; a `Marchable` trait with a
singular `continue_with`; named-stage coupled march building; event-fork counterfactual campaign
verbs; and a two-round refine machinery. All CFD study examples were migrated onto it.

**Formal verification begins** (#673, July 5). Lean 4 and Kani were set up with a scaffold, then the
crates were formalized in sequence: haft with mirrored Rust witnesses (July 5), the causal arrow laws
and `CausalMonad` congruence (July 6 and 7), `deep_causality_core`'s carrier stack, fold universality
and relay termination (July 7), the extracted num, algebra, complex, and dual crates (July 8), and
the main crate's graph algebra, causaloid-layer theorems, the Context hypergraph, and the keystone
result that `evaluate` is the unique catamorphism per fixed carrier (July 10). Lean proofs were added
to the Bazel build on July 16 (#709), and a proof map was published at `/formalization/` on the docs
site on July 12 (#697).

**Categorical machinery in haft** (#675, July 8). `Foldable::fold_map`, a named `Category` and
`Kleisli` where composition is bind, a reified free Arrow (`ArrowTerm`), a one-way interpreter from
`ArrowTerm` to `Kleisli<M>`, and a symmetric-monoidal PROP. Capability traits followed: `EqFunctor`
and `DebugFunctor` with opt-in `Eq`/`Debug` for `Free` (#705, July 14), and `CloneFunctor` (#719,
July 24).

**The num crate split.** `deep_causality_num` was split into a layered tower and three new crates
were first published to crates.io on July 8: `deep_causality_algebra`, `deep_causality_num_complex`,
and `deep_causality_num_dual`. `deep_causality_file` was published the same day.

**`deep_causality_quantum`, a new crate** (#694, July 10 to 12). Quantum-information kernels moved
out of physics into their own crate: density matrices, channels, and gates over
`CausalTensor<Complex<R>>`, with a flat quantum causal model (QCM). Not yet published to crates.io.

**Supersonic retropropulsion** (July 17 to 20). A propulsion kernel family in physics, a de-risking
study of the plume-coupling seam that returned an AMBER verdict, then four milestones: burn-envelope
contracts with an inheritance guard, coupled retro-thrust and plume-obstruction stages, terminal
descent with guidance and live envelope enforcement, and the M5 example running from blackout exit to
touchdown. Typed witnesses record commits, re-seeds, transitions, and peak bond dimension.

**The CFD project website** launched on July 20, with a "Why" page mapping the crate to the standing
challenges in the field added on July 21.

**CFD physics and evidence:** DEC scalar transport with a real Fourier-law wall heat flux (July 22),
an enforceable verification evidence layer (July 21), and the QTT solver's numerical envelope closed
(July 23).

**Build:** the Astro website build was added to the Bazel configuration on July 19 (#714).

**Documentation:** `SKILLS.md`, a user-facing agent guide for building with deep_causality (#693).

## August 2026

**BuildBuddy remote build execution, end to end** (August 11 and 12). Module sources vendored, the
hermetic LLVM version tag fixed, all code examples added to the Bazel configuration (#727), and the
GitHub test action replaced by a BuildBuddy CI workflow (#729). Library-crate example binaries now
build and run under Bazel.

**`rules_lean` lifted into its own rule set** (#731), aimed at publication on the Bazel Central
Registry, and later moved to its home on bazelverse.

**`deep_causality_cfd` 0.1.0 published to crates.io** on August 12, along with the release of
`deep_causality` 0.15.1 and `deep_causality_physics` 0.8.1.

**The cup product in topology** (#736, August 21). Generic over `ChainComplex`, giving the cohomology
ring structure rather than only the groups.

**`no_std` support across 18 crates** (#738, August 21).

**The numeric tower completed** (#740, August 23 and 24). All five number systems now have a place:
`NaturalNumber` for N, the integers admitted into the algebra tower, and `deep_causality_num_rational`
for Q, published to crates.io on August 23. The algebra tower gained a semiring level, an
`IntegralDomain`, a finite field with an admission guard, morphisms with a domain and a codomain, and
each algebraic law restated about the operation it governs. The Euclidean domain and Q were
formalized in Lean.

**`deep_causality_linear`, a new crate** (August 24 and 25). Sparse CSR, dense, and bit-packed F2
matrices and vectors; eliminations, decompositions, conjugate gradient, and an exact integer path.
The workspace was migrated onto it and the duplicated linear algebra in the topology crate was
retired. `deep_causality_sparse` became a re-export shim. Not yet published to crates.io.

**The quantum crate sub-site** was published on August 25.

**CI change:** the Miri workflow was removed on August 20 because of GitHub Actions minute
consumption, and the slow CFD verification harnesses were moved from a nightly to a monthly schedule
(#739).

---

## New crates in 2026

| Crate | In repository | crates.io |
|---|---|---|
| `deep_causality_calculus` | June 4, 2026 | 0.1.0, 2026-06-08 |
| `deep_causality_fft` | June 12, 2026 | 0.1.0, 2026-06-17 |
| `deep_causality_par` | June 12, 2026 | 0.1.0, 2026-06-17 |
| `deep_causality_cfd` | June 15, 2026 | 0.1.0, 2026-08-12 |
| `deep_causality_file` | June 30, 2026 | 0.1.1, 2026-07-08 |
| `deep_causality_algebra` | July 8, 2026 | 0.1.0, 2026-07-08 |
| `deep_causality_num_complex` | July 8, 2026 | 0.1.0, 2026-07-08 |
| `deep_causality_num_dual` | July 8, 2026 | 0.1.0, 2026-07-08 |
| `deep_causality_quantum` | July 10, 2026 | not yet published |
| `deep_causality_num_rational` | August 23, 2026 | 0.1.0, 2026-08-23 |
| `deep_causality_linear` | August 24, 2026 | not yet published |
