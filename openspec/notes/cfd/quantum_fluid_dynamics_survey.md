# Quantum Fluid Dynamics — SOTA survey, scoped to the DeepCausality CFD stack

**Status.** Exploratory survey, 2026-06-16. A literature scan plus an honest mapping onto what
the repo already demonstrates, written to sharpen which "quantum fluid dynamics" strands are
worth tracking or prototyping and which are hype.

**Companion notes.** Reads against `causal_cfd.md` (§0 value proposition, the three capabilities
this survey shows are unique), `cfd-gap.md` (the DEC core: wedge, interior product, de Rham/♯
isos, `leray_project`, `hodge_decompose`), and `cfd-roadmap.md` (sequencing). New external
citations sit in §6 and should be folded into `references.md` rather than duplicated there.

**Method.** Five-angle web fan-out: 22 primary sources fetched, 102 falsifiable claims
extracted, top 25 put through 3-vote adversarial verification (23 confirmed, 2 killed). Every
paper cited was actually fetched. Claim-level provenance is in the run transcript
(`wf_c3836088-fa2`). The prose reconstructs the verified claim ledger. The claims are verified;
the connective narrative is editorial. Everything that failed verification, or that is hype
relative to its marketing, sits at the very end in §7. The body above carries only what holds up.

---

## 0. The one-paragraph answer

Quantum advantage for CFD does not exist today. It is gated on fault-tolerant hardware that does
not exist, and several authors in the corpus concede this in their own words, so the
quantum-algorithm strand (HHL/VQA/QLBM/Carleman) is monitor-only. Of the rest, exactly one is
near-term classical engineering: **tensor networks** (MPS/QTT), which compress turbulent fields
on today's hardware and reuse the repo's tensor/FFT stack. Two more map onto math the repo
already ships: the **Schrödinger/Madelung** flow, whose `U(1)`-phase/Hopf-fiber spinor
representation is literally the `HopfState` example, and **structure-preserving DEC + geometric
algebra**.

The decisive finding came from auditing the code, not the literature. The physics and topology
crates ship a near-complete, validated gauge-theory layer: a continuum `GaugeField` for
`U1…SU3×SU2×U1` and `SO3_1`, plus a 24-test-validated `LatticeGaugeField` with Wilson/Polyakov
loops, Metropolis, and Wilson flow. All of it sits over the same topological substrate as the
DEC CFD solver. This retires the earlier "GA⊕DEC unification is unexplored" caveat, since the
connection, curvature, and holonomy primitives already exist and are tested for gauge theory. It
also names a novel, repo-aligned direction the literature does not pursue: **gauge-theoretic /
holonomy CFD**, where velocity is a `U(1)` connection, vorticity is its curvature, and
circulation is a Wilson loop conserved by construction (exact discrete Kelvin's theorem, Strand
4.5). That direction needs no hardware that does not exist.

A second finding is orthogonal to "quantum" altogether: the entire QFD literature lacks
counterfactual, corrective, and uncertainty-typed composition. The repo already demonstrates all
three (`quantum_counterfactual`, the `corrective_*` interventions, `MaybeUncertain` inflow),
which is the DeepCausality moat in `causal_cfd.md §0`. Net: do not chase quantum advantage.
Prototype tensor-network compression and the Schrödinger-flow marcher, spike gauge-theoretic
CFD, and own the causal/counterfactual/uncertain layer the field has not built.

**The joint accuracy-and-speed metric that decides everything is wall-clock to within 1–2% of
the published reference. It, and the resulting implementation order, are in §0.5. Read that
first.**

---

## 0.5 The deciding metric — wall-clock to target accuracy — and what gets implemented

This section is the operative conclusion of the whole note. Everything else is supporting
analysis.

**Definition.** "Done" does not mean the residual stopped changing. It means the computed
observable matches the published reference within 1–2% (cylinder C_d ≈ 1.32–1.34 and St ≈
0.164–0.166 at Re=100; the Taylor–Green energy-decay history). Formally:

```
T_solution  =  minimum wall-clock such that  |observable − reference| ≤ 1–2%
```

This couples accuracy and speed into a single number, and that coupling is why CFD is hard. In
classical engineering, accuracy and performance are near mutually exclusive, so they must be
optimized in lockstep rather than traded. Two consequences reorder everything.

First, convergence is not accuracy. A scheme converges to its own discrete solution, a fixed
discretization error away from the true PDE. A method that converges fast to an answer 3% off
has `T_solution = ∞` at the 1–2% bar. Speed below the accuracy threshold is worthless.

Second, there are two distinct bottlenecks, and they need different levers:

1. **The accuracy floor.** Can the scheme reach 1–2% at all? This is set by convergence order
   `p`, mesh resolution `h` (error ~ `C·hᵖ`), and structure preservation. A non-conservative
   scheme can sit 3% off forever because vorticity or energy leaked. Levers: higher order,
   conservation, adequate `h`.
2. **The efficiency.** Reach that accuracy in minimum wall-clock, `N_steps × C_step`, at the
   resolution the floor demands. Levers: unconditional-stability or steady-state solves
   (`N_steps`), fast Poisson or compression (`C_step`).

So you co-optimize. Pick the scheme with the best error-per-DOF, so the floor is met at the
fewest DOF, then minimize wall-clock at that DOF count. Only four levers move accuracy and speed
together, and they are the ones that matter for the joint objective:

- **(i) Higher convergence order.** Meets 1–2% at coarser `h`, hence fewer DOF, hence less
  `C_step` and fewer or larger steps. The 1–2% target promotes this to a top lever. An earlier
  revision that over-emphasized `N_steps` alone under-weighted it.
- **(ii) Structure preservation / conservation.** Removes a systematic bias that no amount of
  stepping fixes. It is necessary to land on the reference, not merely near it.
- **(iii) Compression with tolerance ε ≪ target.** Affords fine `h`, and so low discretization
  error, cheaply. This holds only while compression error stays well under the 1–2% budget.
- **(iv) Direct steady-state solve.** Reaches the accurate fixed point without paying for the
  transient. This holds only while the discrete fixed point is itself within 1–2%.

**Re-scored candidates against "hit 1–2% in minimum wall-clock":**

| Candidate | Can it reach 1–2%? (the accuracy floor) | Wall-clock once it can | Joint verdict |
|---|---|---|---|
| **A. Tensor networks (MPS/QTT)** | Yes. ε ≪ 0.01 affords fine `h`, provided compression error stays ≪ 1–2% | Strong: poly-log DOF; DMRG steady-state skips the transient | **Best joint candidate.** Affordability multiplier for a fine, accurate grid. Risk: χ-blowup eating the accuracy budget in 3D |
| **B. Schrödinger / ISF** | Doubtful at 1–2% on viscous flow. The `ℏ` artifact and imaginary-diffusion hack are vis-grade, never validated to C_d/St tolerance | Fast (big steps), but irrelevant if it misses the bar | **Demoted.** Fails the floor on the project's viscous reference; coarse-propagator only |
| **C. DEC + GA** | Yes, if 2nd-order (structured/graded) plus conservation. 1st-order unstructured needs punishing DOF | Low `C_step`; explicit means many steps (fix via implicit) | The substrate that can land on reference. Cost is DOF-at-low-order. **Raising effective order is on the critical path** |
| **D. Gauge / holonomy** | Possibly lowers the St floor: exact circulation gives correct shedding at coarser mesh. No help for C_d | ≈ C | **Accuracy lever for St specifically**, unproven. Not a speed play |

**Key realization.** Under the 1–2% target, the dominant both-axis levers are convergence order
and conservation. They decide whether you reach the reference at all and at how many DOF.
Compression is the affordability multiplier; implicit and steady-state solves mop up the
remaining wall-clock. A fast scheme that misses the bar (ISF on viscous flow) scores
`T_solution = ∞`, whatever its step count.

### The decision — implementation order (success metric: wall-clock to within 1–2% of reference)

**Phase 1. Establish the accuracy floor and cut `N_steps`. Both, all in-repo.**

- Verify and raise the spatial order and conservation of the DEC marcher on MMS and
  Taylor–Green. Confirm 2nd-order on structured/graded meshes and machine-precision
  conservation. This sets the DOF needed to hit 1–2%. It is the accuracy floor, and nothing
  downstream matters if it is not met. (It lifts the algebraic-interior-product /
  combinatorial-wedge advection recipe from arXiv:2508.12501 into the `cfd-gap.md` seam.)
- Replace the explicit march with an unconditionally-stable implicit/symplectic integrator
  (arXiv:2402.02905), so the timestep is accuracy-limited, not CFL-limited.
- Add a fast pressure-Poisson solve: FFT (`deep_causality_fft`) on periodic Taylor–Green,
  geometric multigrid on the cylinder (cf. Decapodes' 97.8% RMSE reduction vs GMRES). Add
  Anderson or Newton–Krylov acceleration to skip the transient on steady or periodic cases.
- Success metric: wall-clock to cylinder St/C_d within 1–2%. Not residual, not per-step cost.

**Phase 2. Make the accurate grid affordable, then skip the transient.**

- Build the QTT/MPS field backend with truncation tolerance ε ≪ 1–2%, so compression never eats
  the accuracy budget. MMS-gate on Taylor–Green, then cylinder St/C_d. The laminar-shedding Re is
  the compression sweet spot. Escalate to a DMRG-style steady-state solve in compressed form,
  where A stops being a per-step win and becomes an `N_steps` win. This is the A×C fusion: a
  fine, conservative, high-order field made affordable by compression and solved straight to its
  fixed point.

**Defer or reclassify.**

- ISF: coarse propagator in Parareal only. It does not clear 1–2% on viscous references on its
  own. Harvest its big-stable-step speed and let the DEC fine solve carry the accuracy.
- Gauge / holonomy: evaluate strictly as an accuracy-floor lever for St. Does exact circulation
  reach the shedding frequency at a coarser mesh? It is not a speed play and has no `N_steps`
  lever. Research shelf.

---

## 1. Strand-by-strand assessment

### Strand 1 — Quantum algorithms for Navier–Stokes — *theoretical / NISQ-toy; hype vs. claims*

Quantum linear solvers (HHL and variants), variational quantum algorithms, quantum lattice
Boltzmann (QLBM), Carleman linearization for the nonlinearity.

Verified findings:

- Exponential-speedup claims are conditional on fault-tolerant hardware that does not exist.
  Current devices are noise-limited, forcing shallow variational circuits.
- "Speedups" exclude state preparation and readout. State prep is replaced by an assumed ideal
  oracle that authors themselves flag as a top open problem. One HHL solver concedes no
  end-to-end speedup outright, because reading out 2ⁿ amplitudes forces classical O(2ⁿ)
  handling and forfeits the Hilbert-space advantage.
- Nonlinearity is handled only by Carleman linearization. The whole speedup rests on a contested
  assumption: that low-order Carleman truncation is far more accurate than previously believed.
- Working hybrid schemes quantize only the linear pressure-Poisson solve. Advection and
  diffusion stay classical. These are not quantum NS solvers.
- Variational quantum algorithms are "unlikely to beat optimal classical methods beyond a
  certain circuit depth on noisy devices."

One headline claim in this strand, IBM-hardware execution of a variational NS solver, was
adversarially killed. See §7.

Repo relevance: near-zero for roughly 5–10 years. No classical port, and no in-repo substrate,
correctly so for a classical library. The one durable takeaway is negative and useful. The
readout bottleneck is exactly what kills naive quantum CFD, which is the structural argument for
Strand 2. **Action: monitor only.**

### Strand 2 — Tensor-network / quantum-inspired CFD — *demonstrated, classical, prototype-worthy* ⭐

The only near-term-actionable strand. Matrix product states (MPS) and quantized tensor trains
(QTT) compress turbulent fields because turbulence has limited inter-scale "entanglement." The
methods are fully classical and use no quantum hardware.

Anchor papers (all fetched):

- **Gourianov et al., *Nature Computational Science* 2022** (doi s43588-021-00181-1). The
  foundational MPS-turbulence paper (Jaksch, Kiffner, Lubasch, Givi, Babaee). It establishes the
  inter-scale-entanglement compression mechanism.
- **Kiffner & Jaksch, arXiv:2303.03010.** MPS incompressible NS, memory and runtime
  poly-logarithmic in mesh size, roughly an order-of-magnitude runtime gain over DNS. Reaches
  Re ≈ 10⁷, where DNS grid cost explodes.
- ***Communications Physics* 2024** (doi s42005-024-01623-8). MPS lid-driven cavity. The bond
  dimension grows only logarithmically in time. Validated against DNS.
- **arXiv:2407.09169.** Tensor-train turbulence PDF method (5+1-D reactive turbulence): O(10³)
  cost reduction vs finite-difference, runs on a single CPU core. (A widely-quoted "10⁶" version
  of this figure was adversarially killed. See §7.)
- **arXiv:2508.12191** (Aug 2025). MPS for Gross–Pitaevskii: 10× to 10,000× compression,
  reproduces the incompressible kinetic-energy spectrum, 12.1× GPU speedup via cuQuantum. Bridges
  Strands 2 and 3.

Hard limits, verified and not hype: the bond dimension is χ = O(poly(1/ε)); compression is
proportional to vortex or soliton density, so the advantage degrades exactly where you most want
it, in dense 3D turbulence; the demonstrations are 2D, laminar, and short-time; the residual
bottleneck is the Poisson solve. Authors uniformly frame this as a stepping stone to quantum
CFD. The quantum part is aspirational. The demonstrated win is entirely classical.

Repo relevance: direct and high. `deep_causality_tensor` (einsum), `deep_causality_sparse`
(CSR), and `deep_causality_fft` already exist. An MPS/QTT velocity-field representation is a
compressed tensor backend behind the `CfdScalar` field abstraction. It composes with the
existing einsum and would slot under the DEC marcher as a storage and operator layer.
**Action: prototype** a QTT field backend on a periodic Taylor–Green case, MMS-gated like
everything else.

### Strand 3 — Schrödinger / Madelung "quantum hydrodynamics" of classical fluids — *parts already in-repo* ⭐

A classical algorithm that borrows the quantum formalism. It lands directly on machinery the
repo already ships.

Anchor papers (fetched):

- **Chern, Knöppel, Pinkall, Schröder et al., "Schrödinger's Smoke," SIGGRAPH 2016** (UCSD/TU
  Berlin). The fluid is a complex-vector-valued wavefunction under a constrained Schrödinger
  equation. The integrator is operator splitting with FFT, for both the Schrödinger step and the
  incompressibility projection. It reproduces vortex wakes and filament interaction without
  Lagrangian tracking and without a vorticity-loss correction. The Incompressible Schrödinger
  Flow (ISF) line derives from a Landau–Lifshitz-type Hamiltonian, which is structure-preserving,
  and introduces a discrete Lie derivative on differential forms of arbitrary degree, an operator
  shared with DEC.
- **Meng & Yang, *Phys. Rev. Research* 5, 033182 (2023).** The Hydrodynamic Schrödinger
  Equation: Madelung generalized to flows with finite vorticity and dissipation, as unitary
  evolution of a two-component (spinor) wavefunction. Turbulence appears as tangled vortex tubes
  with Kolmogorov −5/3 scaling.
- **Meng & Yang, *Phys. Rev. Research* 6, 043130 (2024)** (arXiv:2403.00596). An exact mapping of
  Navier–Stokes to a nonlinear two-component Schrödinger–Pauli equation (a non-Hermitian quantum
  spin system).
- **arXiv:2401.11149.** A superfluid (GP) quantum vortex tangle used as a "skeleton" to
  synthesize classical turbulent fields that reproduce classical statistics.

**Why the codebase upgrades this strand, not just the literature.** The
`hopf_fibration_multivector` example
(`examples/quantum_examples/hopf_fibration_multivector/main.rs`) already implements
`HopfState::from_spinor(α, β).project()` to the Bloch sphere with an invisible `fiber_shift`
(global phase). That is the ISF representation: a 2-component ℂ² spinor, velocity recovered
through a Clebsch/Hopf map, with the gauge or global-phase freedom being the Hopf fiber. The
remaining ISF pieces are also present:

- the spinor or wavefunction carrier: `HilbertState`, `HopfState`, `CausalMultiVector`;
- FFT split-step evolution: `deep_causality_fft`;
- the incompressibility projection: the DEC Leray `leray_project` and `hodge_decompose` entry
  points assumed available per `cfd-gap.md`.

Maturity: ISF is a demonstrated computational scheme for visually plausible vortex-dominated
incompressible flow, of graphics and visualization provenance. It is not a quantitatively
validated NS solver. The honest framing for us is a vortex-preserving alternative marcher,
attractive because it conserves circulation by construction and needs no vorticity-confinement
hack. It must still clear the same MMS, Taylor–Green, and cylinder verification bar as the DEC
marcher before any claim. The quantum-hardware angle (Meng & Yang's Qiskit-simulator runs)
carries no demonstrated advantage.

Repo relevance: high, and the tightest architectural fit. This strand most fully exercises the
UNIFORM_MATH "Topology → Tensor → Algebra → Effect" cycle on a real fluids problem, reusing the
quantum kernels that already exist rather than adding new math. **Action: prototype** ISF as an
`Operator` in the `CfdFlow` DSL behind a feature, and compare vortex preservation against the DEC
marcher on the shedding-cylinder case.

### Strand 4 — Structure-preserving DEC + geometric algebra / lattice gauge — *foundation; partly validated in-repo* ⭐

Not "quantum," but the repo's mathematical home, and the corpus confirms it is an active,
validated SOTA line.

Verified findings:

- DEC gives a complete discretization of incompressible NS on simplicial meshes (exterior
  derivative, Hodge star, wedge, interior product). It is mimetic: curl-of-gradient is zero
  discretely, and mass and vorticity are conserved to machine precision (arXiv:2508.12501). Known
  limit: 2nd-order on structured triangular meshes, 1st-order on unstructured, which bears
  directly on our MMS convergence expectations.
- The nonlinear advection term needs an algebraic discretization of the interior product plus a
  combinatorial wedge. That is a concrete recipe for the DEC advection seam in `cfd-gap.md`.
- **Decapodes.jl / CombinatorialSpaces.jl** (arXiv:2411.13569): the first DEC porous-convection
  simulation. A DEC geometric multigrid gave a 97.8% RMSE reduction vs GMRES, at 42.7% higher
  runtime. Practical, not merely theoretical. The closest external analog; study it as prior art.
- Symplectic and variational integrators for dissipative NS are unconditionally stable via hidden
  Hamiltonian structure (arXiv:2402.02905). Structure-preserving compressible NS via
  Onsager–GENERIC metriplectic splitting comes from the ResearchGate source, of unreliable
  quality; treat it as a lead, not a citation.
- **Geometric (Clifford) algebra for fluids** (*Phys. Fluids* 32, 087111): multivectors for
  vorticity, helicity, and parity, yielding fluid analogues of Maxwell's equations and a Lorentz
  force from a single multivector equation. One verified nuance matters. In the external
  literature, the DEC-fluids and GA-fluids lines are distinct tracks; exterior-calculus fluid
  papers make no use of Clifford algebra. A stack that unifies DEC, GA, and a gauge-connection
  layer under one HKT foundation is therefore uncommon in the published record.

**Correction (2026-06-16): this strand is substantially built and validated here, not
"unexplored."** An earlier revision under-counted the repo. The physics and topology crates ship
a near-complete gauge-theory layer over the same `SimplicialComplex` / `Manifold` / lattice
substrate the DEC CFD solver uses:

- **Continuum gauge fields.** `deep_causality_topology::GaugeField<G, M, R>`: a connection 1-form
  `A` and a curvature 2-form via the structure equation `F = dA + A∧A` (non-abelian) or `F = dA`
  (abelian). Parameterized over `GaugeGroup` markers `U1, SU2, SU2_U1, SU3, SU3_SU2_U1, SO3_1`
  (`LIE_ALGEBRA_DIM`, `IS_ABELIAN`, structure constants `f^{abc}`).
- **Physics aliases** (`deep_causality_physics/src/theories/alias/mod.rs`): `EM = GaugeField<U1>`,
  `WeakField = SU2`, `ElectroweakField = SU2_U1`, `QCD = SU3`, `SMField = SU3_SU2_U1`,
  `GR = SO3_1`. The EM ops compute E and B, the Poynting vector, the Lorentz force, and both
  Lorentz invariants from `F_μν` (`gauge_em_ops.rs`). The electroweak ops include one-loop
  radiative corrections.
- **GR as an SO(3,1) gauge theory** (`general_relativity/gr_ops.rs`, `adm_ops.rs`,
  `adm_state.rs`): Christoffel as connection, Riemann as curvature (`CurvatureTensorWitness`),
  Ricci, the Ricci scalar, Kretschmann, and ADM 3+1 extrinsic curvature. That is a
  numerical-relativity slicing layer, directly relevant to relativistic fluids and GRMHD on
  curved meshes.
- **Lattice gauge theory.** `LatticeGaugeField<G, D, M, R, S>` (Wilson formulation, group-valued
  link variables `U_μ(n)`), with a 24-test physics validation suite per the module header: 2D
  U(1) exact `I₁/I₀`, strong and weak coupling limits, Wilson loops (`try_wilson_loop`), Polyakov
  loops (`try_polyakov_loop`), plaquette and rectangle (`try_plaquette`, `try_rectangle`),
  Symanzik/Iwasaki/DBW2 improved actions, gauge-invariance checks, topological-charge detection,
  and Metropolis Monte-Carlo thermalization (`ops_metropolis`, `ops_monte_carlo`), plus gradient
  (Wilson) flow and smearing (`ops_gradient_flow`, `ops_smearing`). The `S` type-parameter is a
  source type-state (vacuum `()` vs. sourced) carried through `with_source`.

So the holonomy primitives (Wilson and Polyakov loops, plaquettes, gauge transforms) and the
curvature primitives (continuum `F = dA`, Riemann) already exist and are validated for gauge
theory. What is not yet built is the binding of these to a fluid velocity field (see §4.5). That
is a far shorter hop than "unexplored." The honest caveat stands: the suite is validated against
Creutz lattice-gauge benchmarks, not against fluid problems.

Repo relevance: foundational. Actionable items: adopt the algebraic-interior-product /
combinatorial-wedge advection recipe; document the 1st-order-on-unstructured-mesh behavior in the
MMS suite; study Decapodes as the reference implementation; treat the GA⊕DEC unification as a
research contribution, not a solved technique. **Action: track + harvest recipes.**

### Strand 4.5 — Gauge-theoretic / holonomy formulation of fluid dynamics — *novel, repo-unlocked, unproven* ⭐ (new, 2026-06-16)

The gauge stack above opens a CFD direction that none of the 22 surveyed sources pursues, and
one the repo is unusually well-positioned for: treat incompressible flow itself as a `U(1)`
gauge field. The dictionary is exact:

| Fluid object | Gauge object | Repo primitive |
|---|---|---|
| velocity 1-form `u` | connection `A` | `GaugeField<U1>` connection slot / link variables |
| vorticity 2-form `ω = du` | curvature `F = dA` | abelian field strength (`compute_field_strength_abelian`) |
| circulation `Γ = ∮_C u` | Wilson loop / holonomy | `try_wilson_loop`, `try_polyakov_loop` |
| Kelvin's circulation theorem | gauge-invariance of holonomy | gauge-transform invariance tests |
| helicity `∫ u∧du` | Chern–Simons-type invariant | lattice topological-charge `Q` |
| ISF / Madelung phase freedom | `U(1)` fiber on a matter field | `HopfState` fiber + `HilbertState` |

Why this is worth a line in the deck and not just trivia:

- It is a structure-preserving integrator argument. A holonomy-conserving update keeps the
  gauge-invariant, that is, circulation, exact to machine precision (discrete Kelvin's theorem).
  This is the property classical CFD fights to approximate, since vorticity-confinement hacks
  exist only because grid schemes leak circulation. One caveat, corrected 2026-06-16: the repo's
  lattice integrators (`ops_metropolis`, `ops_monte_carlo`, gradient/Wilson flow) are Euclidean
  equilibrium samplers, not time-marchers. Using them for a deterministic fluid march is a
  category error. What is reusable is the data structures (link variables) and the measurements
  (`try_wilson_loop` and `try_plaquette` give circulation and vorticity). A deterministic
  real-time marcher (Kogut–Susskind or symplectic Hamiltonian evolution of the connection) is
  new code, not free reuse of the validated suite.
- The cost is not the QCD-style burden the name suggests. The heaviness reputation belongs to
  non-abelian gauge theory (SU(N) matrix links plus the `A∧A` self-interaction) with fermions
  (Dirac-operator inversion) sampled by Monte Carlo. None of that is present here. Fluid
  vorticity is abelian U(1): a link is a single phase θ (one real), `F = dA` is the plaquette sum
  `θ₁+θ₂−θ₃−θ₄` (three adds, cheaper than a finite-difference curl), no fermions, no
  path-integral sampling. By de Rham, abelian-U(1) holonomy on a lattice is DEC exterior calculus
  on real-valued cochains: link circulation is the Whitney 1-form velocity, plaquette holonomy is
  the DEC curl and so the vorticity, and the Wilson loop is the discrete circulation integral.
  The per-step cost therefore equals the structure-preserving DEC solver the crate already
  targets. The dominant expense in either framing is the same pressure-Poisson / Leray
  projection, which the gauge framing neither adds to nor removes. Stay U(1). Never reach for the
  non-abelian `GaugeField<SU2/SU3>` here; that would be QCD-heavy and pointless for fluids.
- The genuine incremental machinery is the compact-vs-non-compact choice. Non-compact U(1)
  (angles without mod-2π) is plain classical DEC. Compact U(1) admits quantized topological
  vortices (BKT and monopole physics), which is the superfluid and Schrödinger-flow appeal:
  topological reconnection, no surgery. Still abelian, still scalar phases, still far below QCD,
  and relevant only if you want quantized circulation. The repo's topological-charge detection is
  the compact case.
- It is the unifying bridge between the survey's three real strands. ISF/Madelung (Strand 3) is a
  `U(1)` gauge field coupled to a Schrödinger matter field; the GA-fluids "fluid Maxwell
  equations" (Strand 4, *Phys. Fluids* 32, 087111) are that `U(1)` structure; and the DEC
  `F = dA` curvature is the shared discretization. One substrate, three strands.
- It is GR-ready. With `GaugeField<SO3_1>` plus ADM, the same formulation extends to relativistic
  fluids and GRMHD on curved manifolds. The `grmhd` example already couples GR and MHD
  monadically.

Maturity: theoretical and novel. No external paper in the corpus builds CFD on a lattice-gauge
holonomy substrate, and the repo's gauge suite is validated for gauge theory, not fluids. The
unproven, load-bearing step is binding `u ↦ A` and showing a deterministic holonomy-conserving
update reproduces an incompressible NS march at the right MMS order. The honest risk: the
lattice-gauge action (Wilson) is not the NS dynamics. Circulation conservation is necessary, not
sufficient. Viscous dissipation is non-conservative and sits awkwardly in a gauge-invariant
action, and the pressure projection still dominates cost and still must be done. So the payoff,
exact Kelvin's theorem plus clean topological vortices, is regime-limited to inviscid or
weakly-viscous, circulation-dominated flow. This is a research bet, ranked accordingly below. It
is also the single most repo-aligned original contribution the survey surfaced, at
DEC-equivalent cost, and unlike "quantum advantage" it needs no hardware that does not exist.

### Strand 5 — Category-theoretic / monadic compositional simulation — *real but narrow; corrected*

Skeptical brief, honest verdict: real but narrow. One implemented ecosystem, not a movement, and
the "quantum" connection is thin.

- **AlgebraicJulia** (Catlab.jl nearing v1.0, Decapodes, CombinatorialSpaces) is a genuine
  applied-category-theory stack. It compiles string diagrams into executable PDE solvers via an
  operad of wiring diagrams, on a DEC backend, active through 2025. Its geometric multigrid is
  built on a literal functor M: GraphMap → Vect with M(g∘f) = M(g)·M(f), so category theory is
  load-bearing in a real solver.
- arXiv:2401.17432 (math.NA, Jan 2024): DEC/FEEC-generated solvers producing results consistent
  with SU2.
- Not found: any verified result connecting ZX-calculus to fluid simulation, or any
  causal/Kleisli-monad fluid solver literature. Categorical quantum mechanics (ZX) and
  compositional PDE solving (AlgebraicJulia) are separate communities that do not meet over
  fluids.

**Decapodes vs. DeepCausality, corrected.** "Decapodes is essentially DeepCausality's idea in
Julia" is half true. The two share a compositional-DEC and category-theory skeleton. But
Decapodes is a forward PDE compiler only. It has no counterfactual reasoning, no
`intervene`/correction semantics, no first-class uncertainty, no quantum-geometry kernels in the
same algebra, and no precision-genericity. DeepCausality's multiphysics-coupling seam
(`CoupledField` and `Coupling` in `deep_causality_cfd/src/types/flow/coupling.rs`, demonstrated
end-to-end in `multi_physics_pipeline` and `grmhd`) covers the same compositional ground with
static dispatch and type-checked stage tuples instead of dynamic string diagrams. The overlap
validates the architecture. The delta, the causal/counterfactual/uncertain layer, is the part
that is actually about causality, and it is the moat. **Action: do not market "category theory +
CFD" as our unique quantum idea; market the causal layer.** Track AlgebraicJulia as prior art,
and as a credibility anchor that compositional DEC is sound.

---

## 2. The cross-cutting finding the survey actually surfaced

None of the 22 sources, across quantum hardware, tensor networks, Schrödinger flow, DEC, and
category theory, does counterfactual, corrective, or uncertainty-typed fluid computation. The
repo already demonstrates all three primitives, and they compose:

- **Counterfactual / post-selection over state history.** `quantum_counterfactual` rewinds a
  `QuantumHistory` of `HilbertState`s on a detected syndrome
  (`examples/quantum_examples/quantum_counterfactual/main.rs`). The CFD DSL exposes the same via
  `*_with_config` overrides for counterfactual marches (`march_run.rs`).
- **Corrective closed-loop control as a type property.** `Intervenable::intervene` replaces an
  in-flight value with an `EffectLog` audit entry. The five `causal_correction_examples` show
  open-loop failure vs. closed-loop rescue on the same bind-chain. This is `causal_cfd.md §0`'s
  "structured library of corrective interventions" (CFL adaption, divergence rescue, BC fallback)
  as named, logged, testable units.
- **Selective probabilistic typing of the field.** `MaybeUncertain<R>` inflow zones
  (`deep_causality_cfd/src/solvers/dec/uncertain_inflow/`) and the `causal_uncertain_examples`,
  which is `causal_cfd.md §0` capability one.

This is the defensible position. DeepCausality should not chase quantum advantage in CFD. It
should own counterfactual, corrective, and uncertainty-typed compositional CFD, which the QFD
field has not built and which, per `causal_cfd.md §0.1`, is structurally hard for incumbents to
retrofit.

---

## 3. Ranked shortlist (sharpened by what the repo already ships)

This table is the broader strategic ranking, by capability and novelty. The
accuracy-and-speed implementation decision is in §0.5. Under the "wall-clock to within 1–2% of
reference" metric, the top priority is not in this table at all. It is the spatial
order/conservation verification plus implicit integrator plus Poisson plus acceleration work
(Phase 1) that establishes the accuracy floor and cuts `N_steps`. Read §0.5 as authoritative for
what gets built first; read this table for how each strand ranks as a capability.

| Rank | Strand / item | Maturity | In-repo substrate | Action |
|---|---|---|---|---|
| **1** | Tensor-network / QTT field backend (Strand 2) | Demonstrated (classical) | tensor (einsum), sparse (CSR), fft | **Prototype** a QTT velocity-field backend on periodic TG; MMS-gate |
| **2** | Incompressible Schrödinger Flow marcher (Strand 3) | Demonstrated (vis-grade) | `HopfState`/`HilbertState`, fft, DEC `leray_project` | **Prototype** as a `CfdFlow` `Operator` behind a feature; vortex-preservation vs DEC marcher; MMS-gate |
| **3** | Structure-preserving DEC + GA / lattice gauge (Strand 4) | Mature; validated here for gauge theory | `GaugeField`, `LatticeGaugeField` (24 tests), Wilson/Polyakov loops, QGT/Berry, GA multivectors, Regge, ADM | **Track + harvest**: adopt the algebraic-interior-product/combinatorial-wedge advection recipe; study Decapodes |
| **4** | Gauge-theoretic / holonomy CFD (Strand 4.5): vorticity = curvature, circulation = Wilson loop | Theoretical / novel; repo-unlocked | `GaugeField<U1>` data plus `try_wilson_loop`/`try_plaquette` measurements, `HopfState` (marcher is new code, not the MC samplers) | **Research spike**: bind `u ↦ A`, write a deterministic real-time holonomy-conserving update, show it hits MMS order; abelian-U(1) ≈ DEC cost, no hardware needed |
| **5** | Counterfactual + corrective + uncertain CFD (cross-cutting; the moat) | Demonstrated here, absent in the field | `Intervenable`/`EffectLog`, `MaybeUncertain`, `*_with_config` | **Position / emphasize**; align with `causal_cfd.md §0` |
| **6** | Category-theory-as-novelty | One real system (Decapodes) | `Coupling`/`CoupledField` | **Down-rank as novelty**; claim the causal layer, not the compositional-DEC layer |
| **7** | Quantum algorithms for Navier–Stokes (Strand 1) | Theoretical / NISQ-toy | none (by design) | **Monitor only** |

---

## 4. Concrete next steps (ordered to match the §0.5 decision)

0. **Phase 1: establish the accuracy floor and cut `N_steps`. Do this first.** (a) Verify and
   raise the spatial order and conservation of the DEC marcher on MMS and Taylor–Green. Confirm
   2nd-order on structured/graded meshes and machine-precision conservation. This sets the DOF
   needed to hit 1–2% and is the precondition for everything; a fast scheme that cannot reach the
   bar scores `T = ∞`. (b) Replace the explicit march with an unconditionally-stable
   implicit/symplectic integrator (arXiv:2402.02905). (c) Drop in a fast Poisson solve, FFT on
   periodic TG and geometric multigrid on the cylinder, and profile its `C_step` share. (d) Add
   Anderson or Newton–Krylov acceleration on the steady or periodic benchmark. **Success metric:
   wall-clock to cylinder St/C_d within 1–2% of reference.** Not residual, not per-step cost.
1. **QTT field backend spike** (Strand 2, Phase 2): represent a 2^k periodic Taylor–Green
   velocity field as a quantics tensor train. Measure bond-dimension growth and reconstruction
   error vs the dense DEC field. Then escalate to a DMRG steady-state solve (the `N_steps` win)
   and decide whether it earns a place under the marcher.
2. **ISF marcher spike** (Strand 3): wire `HopfState`/`HilbertState` plus `fft` split-step plus
   `leray_project` into a single incompressible step. Validate circulation conservation and MMS
   order on TG. A/B vortex shedding against the DEC marcher.
3. **Advection recipe adoption** (Strand 4): lift the algebraic-interior-product /
   combinatorial-wedge construction (arXiv:2508.12501) into the `cfd-gap.md` advection seam.
4. **Gauge-CFD research spike** (Strand 4.5): on a 2D periodic lattice, store the velocity 1-form
   as a `GaugeField<U1>` connection (abelian, scalar phases, roughly DEC cost). Confirm
   `try_wilson_loop` reproduces circulation around a contour. Then write a deterministic
   real-time holonomy-conserving update (Kogut–Susskind or symplectic, not the repo's
   Metropolis/Monte-Carlo samplers) and check it conserves circulation to machine precision and
   hits MMS order. Probe whether a viscous and pressure-projection seam grafts on without
   breaking gauge invariance. Decide whether exact discrete Kelvin's theorem buys anything the
   DEC marcher does not, staying U(1) throughout, never SU(N).
5. **Reference fold-in**: migrate §6 citations into `references.md` under a new "Quantum /
   quantum-inspired fluid dynamics" section.

---

## 5. Honest gaps in this survey itself

- The deep-research auto-synthesizer returned a degenerate structured object. The narrative is
  reconstructed from the verified claim ledger (claims verified, connective tissue editorial).
- Coverage of quantum turbulence in superfluid helium experiments, as opposed to BEC/GP
  simulation, was thin. A follow-up angle if that physics matters.
- ISF's quantitative accuracy as an NS solver, as opposed to its visual fidelity, is asserted by
  the graphics literature but not independently verified here. The prototype spike is the only
  honest way to settle it.

---

## 6. New citations (fold into `references.md`)

- **[Gourianov-2022]** Gourianov, Lubasch, Dolgov, van den Berg, Babaee, Givi, Kiffner, Jaksch.
  *A quantum-inspired approach to exploit turbulence structures.* Nature Computational Science 2
  (2022). doi:10.1038/s43588-021-00181-1.
- **[Kiffner-2023]** Kiffner & Jaksch. *Tensor network reduced order models for computational
  fluid dynamics.* arXiv:2303.03010 (2023).
- **[CommsPhys-2024]** *Tensor-network solver for incompressible Navier–Stokes (lid-driven
  cavity).* Commun. Phys. 7 (2024). doi:10.1038/s42005-024-01623-8.
- **[TTPDF-2024]** *Tensor-train parameterization of high-dimensional turbulence PDFs.*
  arXiv:2407.09169 (2024). (Verified cost reduction O(10³); the "10⁶" figure is incorrect.)
- **[MPS-GP-2025]** *Matrix-product-state solver for the Gross–Pitaevskii equation (cuQuantum).*
  arXiv:2508.12191 (2025).
- **[Chern-2016]** Chern, Knöppel, Pinkall, Schröder, Weißmann. *Schrödinger's Smoke.* ACM
  SIGGRAPH 2016. (Incompressible Schrödinger Flow.)
- **[Meng-2023]** Meng & Yang. *Quantum computing of fluid dynamics using the hydrodynamic
  Schrödinger equation.* Phys. Rev. Research 5, 033182 (2023).
- **[Meng-2024]** Meng & Yang. *Quantum spin representation of the Navier–Stokes equation.* Phys.
  Rev. Research 6, 043130 (2024). arXiv:2403.00596.
- **[VortexSkeleton-2024]** *Quantum vortex skeleton for synthesizing classical turbulence.*
  arXiv:2401.11149 (2024).
- **[DEC-NS-2025]** *Discrete exterior calculus discretization of incompressible Navier–Stokes on
  simplicial meshes.* arXiv:2508.12501 (2025).
- **[Decapodes-2024]** *Decapodes / CombinatorialSpaces: DEC porous convection & geometric
  multigrid.* arXiv:2411.13569 (2024). AlgebraicJulia: https://www.algebraicjulia.org/
- **[Symplectic-NS-2024]** *Symplectic/variational integrators for dissipative Navier–Stokes.*
  arXiv:2402.02905 (2024).
- **[GA-Fluids-2020]** *A geometric-algebraic approach to fluid dynamics.* Phys. Fluids 32,
  087111 (2020).
- **[FEEC-Compositional-2024]** *Compositional DEC/FEEC solver generation consistent with SU2.*
  arXiv:2401.17432 (2024).

---

## 7. Falsified, killed, and hype directions (what did NOT survive)

Everything here failed verification or is hype relative to its marketing. It sits at the very end
on purpose: the body above carries only what holds up, and this section is the audit trail of
what was discarded and why.

### 7.1 Adversarially killed claims (3-vote verification; ≥2/3 to kill)

- **IBM-hardware execution of a variational NS solver.** Claim: arXiv:2406.00280's variational
  algorithm was executed on real noisy IBM superconducting hardware (lid-driven cavity) with high
  fidelity. Killed 1-2: simulation only. (Strand 1.)
- **10⁶ turbulence-PDF compression.** Claim: tensor-train parameterization of 5+1-D turbulence
  PDFs cuts memory and compute by O(10⁶) (arXiv:2407.09169). Killed 1-2: the real figure is
  O(10³). The O(10³) result itself stands and is cited in Strand 2. (Strand 2.)

### 7.2 Hype / overclaimed directions (true-but-misleading or unsupported)

- **"Exponential quantum speedup for turbulence."** Conditional on nonexistent fault-tolerant
  hardware; excludes state-prep and readout; nonlinearity unsolved beyond contested low-order
  Carleman. (Strand 1.)
- **"Run on quantum hardware."** Frequently means a Qiskit or state-vector simulator, not a
  physical device. (Strands 1, 3.)
- **Tensor-network "quantum CFD."** The demonstrated wins are 100% classical; the quantum framing
  is aspirational, and compression degrades exactly in dense 3D turbulence. (Strand 2.)
- **"Decapodes ≈ DeepCausality in Julia."** Half true; Decapodes lacks the counterfactual,
  corrective, uncertain, and quantum-kernel layers entirely. (Strand 5.)

### 7.3 Internal overclaims corrected during this survey

Logged for honesty, following the deck's self-correcting convention (cf. `causal_cfd.md §0.2`'s
"That was an overclaim"). These were our errors, caught and fixed in the body above:

- **"Reuse the repo's Metropolis / Wilson-flow integrators as the gauge-CFD marcher."** Wrong:
  those are Euclidean equilibrium samplers, not time-marchers; the gauge-CFD marcher is new code
  (deterministic, real-time). (Strand 4.5; corrected 2026-06-16.)
- **"GA⊕DEC unification is unexplored territory."** Wrong: the connection, curvature, and
  holonomy primitives exist and are validated for gauge theory in-repo; what is unproven is
  binding them to a fluid velocity field. (Strand 4; corrected 2026-06-16.)
</content>
