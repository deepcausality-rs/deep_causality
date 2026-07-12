# Dynamic QCM: The Classical–Dynamic–Quantum Causal Bridge

**From a unified causal substrate to quantum error correction, code/algorithm co-design, and quantum-informed CFD**

Purpose: master document from which individual implementation specifications are derived, sequenced until the open frontier is reached. Status markers throughout: **PROVEN** (inherited from existing law or formalized), **DECIDABLE** (checkable at freeze/compile time), **OPEN** (genuine research obligation), **FORBIDDEN** (made unrepresentable by construction).

---

## 0. Thesis

The EPP, formalized in Lean across haft, num, core, and DeepCausality, is one substrate on which classical causality (Pearl), dynamic causality (EPP native), and quantum causal models (Allen–Barrett–Lorenz–Oreshkov) are all expressible. Classical and dynamic are already landed; QCM landed as a dedicated quantum crate; the do-calculus formalization is under way.

Three consequences follow, in ascending ambition:

1. **The bridge**: causal structure that crosses the quantum/classical divide becomes a single graph in one verified type system — expressible in no other framework, because no other framework has both calculi on one substrate.
2. **The quantum computing unlock**: the topology crate's chain-complex machinery is the native language of quantum error correction (CSS/qLDPC codes *are* chain complexes; boundary maps *are* parity-check matrices). Cohomology-operation logical gates (Hsin–Kobayashi–Zhu 2411.15848; Haruna 2511.15224) become computable objects on the crate, the Betti vector becomes a gate catalogue, and algorithm/code co-design becomes a search problem.
3. **The CFD impact**: quantum content in continuum solvers is currently frozen (precomputed, fitted, offline). The bridge makes quantum models live causaloids with near-zero *coupling* cost, dispatched only where classical closures leave their validity envelope, with the value of the quantum correction causally attributed rather than asserted.

The claim discipline throughout: the EPP does **not** subsume QCM by axiom. It **hosts** it by construction. Subsumption is automatic along the axes the EPP axiom varies (determinism, bivalence, temporal precedence, spacetime); along the theory-of-the-relata axis it is a construction — operator-valued state plus a commutativity check — and stating exactly what had to be added is the stronger, falsifiable result.

---

## 1. Critical Assumptions and Make/Break Tests (front-loaded)

Each assumption is falsifiable. Each test is sized in days-to-weeks, not quarters. Failure of a test kills or redirects its dependent specifications cheaply; that is the point.

| # | Assumption | Test | Break consequence |
|---|---|---|---|
| A1 | Monad law 3 (associativity) holds with **operator-valued (matrix) state** in the arity-5 PropagatingProcess, not only scalar state | Property-test bind associativity with complex-matrix payloads in the state field | "Encapsulation-equals-flat inherited for free" collapses; an explicit proof for operator payloads is owed before any quantum nesting claim |
| A2 | **Partial-trace preservation (the one open theorem)**: if a subgraph's CJ factors pairwise-commute and surrounding factors pairwise-commute, the effective factor after marginalisation commutes with its new neighbours | Instrument the freeze-time commutator walk across the test battery; record when the encapsulation branch fails; failure patterns are the conditions of a conditional theorem, or the counterexample | Unconditional failure ⇒ recursion isomorphism is unusable in the quantum regime; conditional ⇒ freeze refuses to encapsulate where the condition fails (subgraph stays flat) rather than aborting |
| A3 | Commutator near-zero tests remain decidable through **nesting depth** (numerical error through iterated partial trace does not swamp the verdict) | Error-propagation analysis; depth-aware tolerance with Float106 (~31 digits); adversarial deep-nesting test cases | Fixed tolerance ⇒ spurious aborts on valid deep models; the causal verdict (commuting = valid) is corrupted by numerics |
| A4 | The topology crate's cell complex can carry (or be extended with) a **global vertex ordering / branching structure** — the prerequisite for cup products | Codebase audit; if absent, design the ordering as a typed property of the complex | No branching structure ⇒ no cup products ⇒ no cohomology-operation gates; this single design question gates the entire QEC track |
| A5 | The RAM-C II blackout corridor is **sensitive to ionization rate-coefficient uncertainty** | Perturb rate coefficients within published uncertainty bands (order-of-magnitude in associative-ionization channels at non-equilibrium); measure movement of blackout onset/exit | Insensitive ⇒ quantum-informed rates buy nothing; CFD quantum track dies at zero cost, redirect to another domain (fusion edge, combustion) |
| A6 | Extracted quantum kernels are **quantum-dynamics-simulation-shaped** (the one advantage class with solid theoretical grounding) and survive dequantization scrutiny | Per-kernel check against classical-simulability literature; one end-to-end resource estimate (state prep, readout, crossover point) for the hardest kernel | Kernel is classically simulable or end-to-end costs consume the advantage ⇒ the advantage-qualification pipeline still works — its output is an honest "no", which is itself the product |
| A7 | Causal decoding beats i.i.d. decoding **under correlated noise** by a measurable margin | Toy surface code + deliberately correlated noise model; causal decoder vs PyMatching; get a number | No delta on the toy model ⇒ the QEC decoding wedge needs reframing before any proposal asserts it |
| A8 | Lean formalization of QCM (process operators, quantum Markov condition, factorisation theorem) is **novel** — verified quantum programs exist (SQIR/VOQC, CoqQ); the causal-model layer appears unformalized | Proper literature check before the claim appears anywhere | Prior art exists ⇒ drop the novelty sentence, keep the artifact |
| A9 | EIC Pathfinder DeepRAP **eligibility structure** (consortium vs single-applicant) permits the intended configuration | Verify call-specific rules first — this determines whether the next months are proposal-writing or partner-finding | Consortium required ⇒ partner search (EU quantum-hardware/QEC group; KCL mathematics as natural UK-adjacent contact) is front-loaded and on the critical path |

Standing terminology guards (zero-cost, permanent):

- **Clifford ≠ Clifford.** The Multivector crate implements Clifford *algebra* (geometric algebra, Cl(p,q)). The gate papers construct gates in the Clifford *hierarchy* (operators normalizing the Pauli group). Different objects sharing a name. The Multivector crate's role is confined to representing Pauli operators; the load-bearing machinery for gates is the topology crate. Never let the name collision into a proposal or a conversation with the formalism's authors.
- **Logical gates ≠ quantum algorithms.** Cohomology operations give the *instruction set* of a code; the *program* is an algorithm (Hamiltonian simulation, phase estimation), which is separate, established discipline. Physics equations do not compile to gates by reformulation; they compile through an algorithm.
- **Fault-tolerance is claim-specific.** HKZ gates are fault-tolerant constant-depth. Haruna's constructions explicitly relax fault-tolerance and depth-optimality. Cite accordingly; QEC reviewers police this distinction.
- **Do-calculus expressibility ≠ identification theory.** Expressing do() as contextual alternation is the easy half; the three rules and Shpitser–Pearl completeness are a genuine formalization project, not "a matter of doing."
- **Physical QPU calls sit in the emergent modality** (§4.9.4): coupling to an open world, outside the formally-verifiable region. Simulated quantum processes (deterministic CJ operators) stay verifiable. State the boundary wherever verifiability is claimed.

---

## 2. The Bridge: Classical → Dynamic → Quantum on One Substrate

### 2.1 Why the hypergraph, not the monad, carries quantum structure

The causal monad's bind is irreversible by construction (`m >>= f >>= g` admits no inverse). Quantum causal structure is symmetric under inversion: U and U† carry the same structure with arrows reversed (Lorenz §6). Therefore quantum causal structure lives in the **causaloid hypergraph**:

- A hyperedge `{Pa(A_i)} → A_i` names the parent set directly — a faithful carrier for the QCM factor ρ_{A_i|Pa(A_i)}, more so than a binary DAG.
- Native non-DAG structure aligns with the QCM frontier: cyclic quantum causal models (Barrett–Lorenz–Oreshkov 2021), indefinite causal order, the quantum switch.
- Lineage: the causaloid roots in Hardy's quantum-gravity work. The EPP dropped Hardy's process matrix for an unconstrained function (implementability, nesting); the Oxford line kept it (and proved the central theorem). Same ancestor, opposite engineering bet.

### 2.2 The carrier: PropagatingProcess monad-state

CJ operators are carried as **state in the arity-5 PropagatingProcess**, threaded via bind — confirmed: hypergraph edge-threading *is* bind. Consequences:

1. The operator is threaded state, not stored context data — matching Lorenz, where the process operator is the object that evolves through the structure.
2. **No shared-mutable-state hazard**: monad-state is threaded, not shared; the order-sensitivity hazard (regime 2) requires shared mutable context and cannot arise.
3. **Encapsulation-equals-flat is monad law 3.** Nested ≡ flat is *associativity* of composition, not commutativity; re-parenthesizing the composition is exactly what encapsulation does. Since edge-threading is bind, law 3 transfers directly. **PROVEN (inherited)** — subject to A1's operator-payload property test.

The only residual context need: genuinely environmental fixed quantum data (e.g. Bell-state preparation ρ_A) lives in **immutable context** — written once at construction, frozen, write methods unreachable. This places QCM in the framework's own verifiable modality (§4.9: static context preserves determinism).

### 2.3 The layer stack

| Layer | Content | Status |
|---|---|---|
| A. Collection order-invariance | `impl Commutative for Collection<C, Agg> where Agg: CommutativeMonoid`; non-commuting configurations rejected by type bound | **FORBIDDEN by construction** — enforceable now |
| B. Encapsulation = flat (skeleton) | Associativity of bind-threading = monad law 3 | **PROVEN (inherited)**, pending A1 |
| C. Shared-context boundary | Encapsulation cuts must not reorder a context write vs dependent read; reachability/cut property over context-dependency edges | **DECIDABLE at freeze** (`freeze_dag`-shaped); does not arise for monad-state quantum case |
| D. Operator commutativity | Quantum Markov condition: factors on overlapping parental Hilbert spaces must pairwise commute (‖AB−BA‖ ≈ 0); free at n=2 by hermiticity, independent and substantial at n≥3 (Lorenz Def. 3.3, fn. 11) | **The only genuinely-quantum check**; enforced at freeze; **OPEN** under encapsulation (A2), numerically gated by A3 |

Layer D is independent of Layer B: associativity makes the encapsulated operator *well-defined*; it does not make it *commute*. The monad proves sound subgraphs compose soundly; it does not prove that collapsing a subgraph via partial trace preserves commutation with the rest. Keeping these separate is the architecture.

Freeze **enforces** Layer D but does not **establish** it: freeze-abort is sound (never accepts an invalid model) but possibly incomplete (may reject valid models where nesting and commutation conflict). The instrumented freeze doubles as the theorem-discovery instrument for A2.

### 2.4 What the completed bridge makes askable (nowhere else well-posed)

- **Hybrid diagnosis**: quantum computers are hybrid systems (classical control, timing, calibration around a quantum core); "is this fault classical or quantum in origin" becomes a query on one graph rather than a debugging session.
- **Hybrid identifiability**: given a graph with both classical and quantum nodes, which interventional/observational data identify which effects — open theory, statable only where both calculi share a substrate and Lean-checked semantics.
- **Quantum do-calculus**: which of Pearl's rules survive at quantum nodes (no observation without disturbance collapses part of rule 2); a Lean-proved conditional statement would be a contribution to both communities.
- **Minimal-quantumness witness**: Wood–Spekkens fine-tuning turned operational — does observed behavior require a quantum node, or is a classical model (without fine-tuning) sufficient?
- **Causal separability at freeze**: does a given process admit any causal order at all — indefinite-causal-order detection as a first-class freeze-time check.

Near-term application with an existing community: **QPU noise diagnostics** — the freeze-time Markovianity check is a device-Markovianity test; direct-cause vs common-cause discrimination for crosstalk/bath coupling changes what the engineer does. The SURD pattern: tool first, theory conversation second (Modi group / process-tensor community).

Honest ceiling: estimating a process operator requires informationally complete interventions at every node — exponential; QCM is confined to few-node systems. The bridge does not repeal this.

---

## 3. The Quantum Computing Unlock: Gates as Topology

### 3.1 The identification

CSS and qLDPC codes are chain complexes; the boundary maps are the parity-check matrices (∂₂ = H_Z^T, ∂₁ = H_X). The toric code is a Z₂ lattice gauge theory. Logical qubits are homology classes; logical operators are non-contractible cycles; **β₁ counts the logical qubits**. The topology crate's chain complexes, Hodge machinery, and Betti numbers are the native mathematics of QEC — machinery almost no QEC simulator carries natively.

### 3.2 The gauge field formalism (operator-valued cochains)

Established lineage (Hsin–Kobayashi–Zhu, arXiv 2411.15848: KCL mathematics / IAS / IBM Quantum; extended by Haruna, arXiv 2511.15224, Osaka): Pauli operators, stabilizers, and logical operators translate to operator-valued cochains (Z_e = (−1)^{a(e)}); logical gates are exponentials of integrated cohomology operations on the gauge fields:

- CZ = (−1)^{∮ a∪a′}; CCZ via triple cup product; C^{n−1}Z via n-fold cup product — each with explicit constant-depth physical-gate decompositions.
- Beyond the color-code paradigm: Steenrod squares, higher cup products, **higher Pontryagin powers** → logical R_k and multi-controlled C^m R_k gates (directly relevant to QFT/phase-estimation/Shor compilation) on projective-space codes.
- Logical action depends only on (co)homology classes — the correctness criterion is homology-class invariance, a **freeze-time check** in this architecture.
- **The Betti vector is the gate catalogue**: qubits at b₁; independently addressable C^{n−q−1}Z generators counted by b_q = b_{n−q}. Given any cell complex, the crate can compute the complete inventory of native fault-tolerant logical gates and their addressability — a static analysis nobody ships.

### 3.3 Gap inventory against the topology crate

Have: chain complexes, cell complexes, Hodge theory, Betti numbers.
Missing (all classical algebraic topology with explicit lattice formulas in the papers):

1. **Cup product with global vertex ordering (branching structure)** — the keystone; gates every construction (A4).
2. Higher cup products (∪_i), Steenrod squares, the explicit higher-Pontryagin-power formula (HKZ App. B).
3. Dual complexes / Poincaré duality.
4. Z_N coefficients with orientation signs (Z₂ shortcuts where −1 = 1 stop working for qudits).
5. Relative (co)homology (boundaries, the folding generalization).
6. **Operator-valued cochains** — the static counterpart of the PropagatingProcess operator-state: same payload type, indexed by cells rather than threaded by bind. One type design, not two.

### 3.4 The QEC wedge and the full stack

General QEC simulation is crowded and mature (Stim, PyMatching, hardware groups). The wedge is what only this substrate can state: **a decoder is a causal-inference engine, and the QEC cycle is a hybrid classical–quantum causal loop.** Syndrome extraction (classical bits) → decoder infers which physical errors caused them (abductive causal inference) → correction as classical intervention on quantum state — under real-time constraints, with noise that is often non-Markovian and correlated (crosstalk, leakage, drift, shared bath). Standard decoders assume i.i.d. errors and degrade exactly where noise has causal structure. The claim: **causally-informed decoding under correlated noise, modeled end-to-end as one verified hybrid causal graph** — the noise's causal structure and the decoder's inference in the same formalism, so "how much logical fidelity does causal noise-modeling buy over i.i.d." is a well-posed, machine-checked question. Stim cannot pose it; it has no causal semantics.

Full-stack demonstration once gates land: define a CSS code on the cell complex → construct logical gates via cohomology operations → run computation under a causally-structured noise model → decode via causal inference → verify homology-class preservation at freeze. Encode–compute–corrupt–correct as **one** hybrid causal graph on one verified substrate.

### 3.5 Algorithm/code co-design (the corrected direction)

Not "physics equations reformulated into gates" — that skips algorithm design, where the difficulty lives. The direction HKZ themselves open: native C^m R_k gates "significantly improve logical circuit compiling" for QFT-heavy algorithms; they call it "searching algorithm-efficient logical gates in quantum codes." The substrate turns this into a search problem it is uniquely equipped for:

> **Given an algorithm's gate-demand profile, search over cell complexes for code topologies whose native cohomology-operation gate catalogue serves that profile cheaply.**

The Betti-vector gate catalogue is the search instrument; the catalogue of a candidate topology is a static computation. Downstream: **verified compilation onto cohomology-operation gate sets** — verified circuit compilation exists (SQIR/VOQC), but this specific composition (algorithm → logical gates whose correctness is a homology-invariance theorem → machine-checked) appears unoccupied (verify per A8-style literature check before claiming).

Sequencing within the track: cup-product primitive → Haruna gates first (general CSS, no FT obligations — validates machinery) → HKZ addressable higher-form gates second (FT, constant-depth — the grant-grade result). Architecture against the general formalism; Haruna as first instantiation.

---

## 4. The CFD Impact: Live Quantum Content, Dispatched and Attributed

### 4.1 The precise problem statement

The defensible claim (not "ionization is nowhere modelled as quantum" — it is, via quantum-derived cross-sections): the quantum content in continuum solvers is **frozen** — precomputed offline, compressed into Arrhenius fits and equilibrium assumptions (Saha, two-temperature), never live or state-resolved inside the solver. Current integration is at best file exchange into semi-quantum solvers; usually not done at all.

Two costs must not be conflated: the bridge collapses the **coupling** cost (the quantum model becomes a causaloid exchanging PropagatingEffects with flow causaloids in one type system) to near zero. It does **not** collapse the **compute** cost (10⁶–10⁹ cells × 10⁴–10⁶ steps × quantum evaluation is prohibitive regardless of coupling). Naive closure replacement is dead on arrival; the feasible architecture follows.

### 4.2 Assumption-gated quantum dispatch (EPP §4.3 + §4.9.3, verbatim)

The classical closure is a causaloid whose **Explicit Assumptions** encode its validity envelope (near-equilibrium populations, T_v/T_tr within fitted range). The assumption test runs everywhere, cheaply. Where assumptions hold → classical closure fires. Where they fail — the strong-non-equilibrium sheath where blackout is decided — a **dispatch causaloid** (Adaptive Reasoning) routes those cells, and only those cells, to the quantum-informed model. "When is the quantum model needed" becomes a computable, per-cell, runtime question instead of a modeling-committee decision made once, offline. Nothing in the standard CFD stack can express this, because nothing in it carries testable assumptions as first-class objects.

### 4.3 Value attribution as a native query

"Where do quantum solvers add real value" is a counterfactual: contextual alternation runs the same corridor twice (classical vs quantum-informed closure) — embarrassingly parallel by §4.12.1 — diffing electron number density, plasma frequency, blackout onset/exit. **SURD** (already ported, fast parallel Rust) then decomposes the causal contribution of the quantum correction into synergistic/unique/redundant components. The value-of-quantum question becomes an information-theoretic attribution with a number, output by the stack rather than asserted by intuition.

### 4.4 Kernel extraction and advantage qualification

Quantum *accuracy* advantage (quantum model reduces error — §4.3's attribution) and quantum *computational* advantage (QPU beats best classical method) are independent claims; a kernel can need quantum physics and be classically simulable (dequantization). The pipeline, each stage derived-and-checked:

1. Causal attribution finds where quantum accuracy matters (SURD, §4.3).
2. Assumption-gated dispatch **extracts the quantum-dynamical kernel** — the small subdomain where physics is irreducibly quantum-dynamical, i.e. the one region where Feynman-class advantage (quantum simulating quantum dynamics) is plausible. Nobody currently has a principled, computable kernel extraction from industrial simulation; here it is an output.
3. Standard algorithm families (Hamiltonian simulation, quantum chemistry) + end-to-end resource estimation (state prep, readout, crossover) answer whether a QPU beats classical *on that kernel*.
4. Co-design search (§3.5) finds the code topology whose native gates compile that algorithm cheaply.
5. The Lean layer verifies the compilation chain.

The only genuinely new methodology is stages 2 and 4 (kernel extraction, catalogue search); the rest is existing practice composed, not claimed. Commercial framing: this is **resource-estimation and use-case qualification tooling** — the demonstrated demand of hardware vendors — differentiated by causal kernel extraction from real simulations and verified compilation. Credibility condition: **the tool must be able to say no.** A methodology that outputs "quantum does not pay here, and here is the causal and resource-level reason" is what makes its positive outputs bankable.

The quantum *content* (rate coefficients, electronic structure) comes from quantum chemistry, not from QCM — QCM's contribution is architectural (hosting, dispatching, attributing). A cloud-QPU-sampling causaloid has its natural first job here (electronic structure for hard non-equilibrium states), with the standing emergent-modality caveat.

Transfer domains once the corridor case lands: fusion edge plasmas (divertor detachment — same atomic-physics-inside-fluid-solver structure), high-enthalpy combustion, planetary-entry aerothermodynamics.

---

## 5. Specification Ladder

Ordered by dependency; each spec is implementable independently once its gate (assumption test) passes. The ladder ends at the open frontier — the items that are research, not engineering.

**Track Q — Quantum crate completion (bridge hardening)**

- **SPEC-Q1** — Operator-valued PropagatingProcess state: complex-matrix payloads in the arity-5 state field; property tests for monad law 3 with matrix payloads. *Gate: A1.*
- **SPEC-Q2** — Freeze-time Layer D check: recursive pairwise-commutator walk over hyperedges sharing a source node; hard pass/abort; instrumented failure-pattern capture. *Feeds A2.*
- **SPEC-Q3** — Depth-aware tolerance: error-propagation analysis through iterated partial trace; tolerance scaling with nesting depth; Float106. *Gate: A3.*
- **SPEC-Q4** — Immutable-context causaloid constructor: read-only context handle, write methods unreachable; home for environmental quantum data (ρ_A).
- **SPEC-Q5** — Layer A type bounds: `Commutative` earned by (container, aggregation) pair; unordered collections legal only under commutative-associative monoids over pure functions.
- **SPEC-Q6** — Causal separability + device-Markovianity diagnostics: indefinite-causal-order detection and non-Markovianity testing as freeze-time first-class operations (the QPU-noise-diagnostics tool; SURD-pattern opener toward the process-tensor community).

**Track T — Topology crate extension (the QEC substrate)**

- **SPEC-T1** — Branching structure: global vertex ordering as a typed property of the cell complex. *Gate: A4. Keystone; blocks T2–T6.*
- **SPEC-T2** — Cup product on simplicial and cubical complexes (explicit lattice formulas per HKZ App. A).
- **SPEC-T3** — Higher cup products (∪_i), Steenrod squares, higher Pontryagin powers (HKZ App. B explicit formula).
- **SPEC-T4** — Dual complexes and Poincaré duality; Z_N coefficients with orientation signs; relative (co)homology.
- **SPEC-T5** — Operator-valued cochains: shared payload type with SPEC-Q1 (one type design — static/cell-indexed vs dynamic/bind-threaded).
- **SPEC-T6** — **Gate catalogue tool**: given a cell complex, compute the full inventory of native cohomology-operation logical gates and addressable generators from the Betti vector; static, freeze-shaped. *(The unshipped tool; also the co-design search instrument.)*

**Track G — Logical gates (on T1–T6)**

- **SPEC-G1** — Haruna gates: S, H, T, (multi-)controlled-Z for general CSS codes as exponentials of gauge-field polynomials; explicit physical decompositions; homology-class-invariance verified at freeze. *(First implementation anywhere, most likely; cite the FT relaxation.)*
- **SPEC-G2** — HKZ gates: fault-tolerant constant-depth C^{n−1}Z via cup products; addressable gates via higher-form symmetries; R_k / C^m R_k via Pontryagin powers on projective-space codes.
- **SPEC-G3** — Full-stack demonstrator: encode → Haruna/HKZ logical gates → causally-structured noise → causal decode → freeze verification, as one hybrid graph.

**Track D — Causal decoding (the grant deliverable)**

- **SPEC-D1** — Toy benchmark: small surface code, deliberately correlated noise, causal decoder vs PyMatching; one honest number. *Gate/produces: A7. Precedes any proposal claim.*
- **SPEC-D2** — Correlated-noise decoding engine: noise causal structure and decoder inference in one graph; logical-fidelity delta vs i.i.d. as the measured claim.

**Track C — CFD quantum dispatch**

- **SPEC-C1** — RAM-C II rate-sensitivity study: perturb ionization rates within published uncertainty; measure blackout onset/exit movement. *Gate: A5. Runs now; needs no quantum crate.*
- **SPEC-C2** — Assumption-gated dispatch: classical closure with Explicit-Assumption validity envelope; dispatch causaloid routing envelope-failing cells to the quantum-informed closure.
- **SPEC-C3** — Counterfactual value attribution: two-context corridor run + SURD decomposition of the quantum correction's causal contribution to blackout prediction.
- **SPEC-C4** — Kernel extraction + one-page resource estimate: hardest collision/electronic-structure computation in the extracted kernel; quantum vs best classical, end-to-end, crossover point. *Gate: A6.*

**Track F — Formalization (the credibility spine)**

- **SPEC-F1** — Lean QCM novelty check and, if clear, standalone write-up (ITP/CPP + Quantum): process operators, quantum Markov condition, factorisation theorem. *Gate: A8. Cheapest publication in the program; work already done.*
- **SPEC-F2** — Do-calculus identification: the three rules; Shpitser–Pearl completeness as the honest-scope target (a project, not an exercise).
- **SPEC-F3** — Quantum do-calculus: which rules survive at quantum nodes, under what commutativity side-conditions; Lean-proved conditional statement.
- **SPEC-F4** — Hybrid identifiability: which data identify which effects in mixed classical/quantum graphs. **OPEN FRONTIER.**
- **SPEC-F5** — Verified compilation onto cohomology-operation gate sets: algorithm → homology-invariance-correct logical gates → machine-checked. **OPEN FRONTIER** (novelty check first).
- **SPEC-F6** — Algorithm/code co-design search over cell complexes against gate-demand profiles, catalogued by SPEC-T6. **OPEN FRONTIER.**

**The open frontier**, precisely: A2 (partial-trace preservation — the one owed theorem), F4 (hybrid identifiability), F5 (verified cohomology-gate compilation), F6 (co-design search), and the advantage-qualification methodology of Track C stages 2+4 as a composed, validated pipeline. Everything below these rungs is engineering gated by week-sized tests.

---

## 6. Strategic Frame (one page, held constant)

- **One deliverable per proposal.** For EIC Pathfinder DeepRAP: *a formally verified causal substrate for hybrid quantum–classical systems, demonstrated on correlated-noise QEC decoding* (Track D), with the Lean formalization as the credibility spine (Karray's formal-ontology framing) and everything else — bridge questions, CFD dispatch, co-design — as horizon narrative, never deliverable. Sprawl is the one-person-shop failure signature reviewers punish.
- **Eligibility first (A9)**, then partner outreach carrying working artifacts (SPEC-D1 number, quantum crate, gate catalogue) — the SURD pattern: tool earns the conversation. Natural contacts: process-tensor/quantum-characterization community (diagnostics); Osaka QIQB (Haruna; Q-LEAP/Moonshot-funded); **KCL mathematics (Hsin)** — author of the formalism being implemented, UK institution, same jurisdiction as the CIC; IBM Quantum adjacency via Zhu.
- **Internal coherence**: chronogravimetric navigation is quantum sensors feeding classical fusion — a hybrid causal system; the bridge is its systems-engineering substrate. The quantum crate existing because the navigation problem needs it is a stronger funding narrative than framework generalization.
- **Negative results are product.** The advantage-qualification tool's ability to output a reasoned "no" is what makes its "yes" bankable — and aligns the program with validation-before-building rather than against it.

---

## 7. Summary

One substrate, three regimes, one owed theorem. The monad threads operator-valued state associatively (law 3, inherited); the hypergraph carries quantum causal structure the bind cannot; the single open quantum proposition is partial-trace preservation of commutativity, enforced sound-but-incomplete at freeze and discoverable empirically through the instrumented walk. On top: the topology crate is the native mathematics of QEC — cup products are the keystone gap, the Betti vector is the gate catalogue, and causal decoding under correlated noise is the wedge no causal-semantics-free simulator can pose. Alongside: quantum content in CFD goes from frozen to live via assumption-gated dispatch, its value causally attributed by SURD, its advantage-plausible kernel extracted rather than asserted. Every rung of the ladder is gated by a falsifiable, week-sized test, and the frontier is named: one theorem, three formalization targets, one composed methodology. Regardless of which gates fail, each failure is cheap, explained, and redirects the ladder rather than collapsing it.
