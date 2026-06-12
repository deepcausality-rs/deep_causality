# Challenge Entry Plan: Competitive Cavity Solver That Demonstrates Four Answers

Status: settled strategy, 2026-06-11 (rev. 2). Decisions:
**no C++** (hard);

**single entry**; the entry must be 

**competitive on the scored metrics** (the
challenge is, in the grand scheme, a low bar) while

**demonstrating four new approaches** to the six open challenges Teschner himself
stated in his blog and works on. Purpose: earn a constructive conversation about
the category of solver the DeepCausality substrate enables, from outside
university affiliation, with the challenge as the credible entry point.

Deadline: **2026-07-07**.

## 1. The posture

Every differentiating property must be *visible in the
artifact's output files or source*, connected to his own stated open problems by
at most one factual sentence each. No manifesto, no platform pitch, no Vision 2030
language in the submission. The artifact does things he has publicly written are
unsolved; he is the one person guaranteed to recognize them unprompted. For an
unaffiliated entrant, the reproducible artifact is the credential.

Compliance discipline: respect every rule except the language — same problem
(lid-driven cavity, Re 1000), same grid (129 nodes, uniform — no graded-mesh
cleverness in the scored run), CFL ≤ 1, single core, his convergence criterion
(max-normalized pressure-residual ratio ≤ 1e-4), exact CSV formats, and his
`evaluate.py` running unmodified via the CMake shim (§4). One deliberate,
disclosed deviation against total compliance elsewhere reads as a position, not
carelessness.

Competitive baseline: second-order central, staggered (MAC) grid, Chorin
projection, deep convergence. Deliberately *Ghia-matching* rather than
truth-matching — the error metric is RMSE against Ghia 1982, itself a
second-order 129-grid solution, so matching its discretization class scores
better than out-resolving it. Speed from a well-tuned pressure solve
(matrix-free CG with a simple preconditioner; geometric multigrid only if week-1
profiling says the Poisson solve dominates embarrassingly). Mid-pack or better on
all three rank columns is sufficient; the cover story must hold on its own.

## 2. The four demonstrated answers

Each maps to one of Teschner's six unsolved challenges (references in
`../cfd/references.md`: Teschner-blog), and each is observable in the submission
itself.

**D1 — Incompressibility as structure, not residual** (his #3, numerical
algorithms). The projected velocity is divergence-free by construction
(projection via matrix-free CG); the solver writes `max |div u|` per iteration
alongside the pressure residual. A reader sees machine-tolerance divergence at
*every* iteration on a metric conventional codes only drive down asymptotically.
One sentence in the README; the rest is a column in an output file.

**D2 — Uncertainty quantification of boundary conditions** (his #3's UQ half —
"quantifying errors and uncertainties with our boundary and initial conditions"
is, in his words, underdeveloped). An optional mode (`--uq`) treats the lid
velocity as `Uncertain` (e.g., ±1% — a real wind-tunnel-grade tolerance),
propagates it by re-running the *identical* chain over sampled draws, and emits
centerline mean ± σ bands plotted against the Ghia points. The scored run is the
deterministic mode; the UQ mode ships in the same binary and is one flag away.
The demonstration: UQ as a type change and a flag, not a bolt-on covariance
framework.

**D3 — In-situ knowledge extraction with provenance** (his #5, plus the direct
bridge to his group's research). Two parts:
  - The converged field passes through a typed extraction stage that detects
    vortex count and centers (Q-criterion + centroid labelling — the shipped
    kernels) and prints them against **Ghia's own vortex-center table** (primary
    vortex + corner eddies at Re 1000) — an accuracy dimension no other entry
    will report, and squarely adjacent to Abolholl, Teschner & Moulitsas (2024)
    on the fragility of vortex-core detection (references.md: Abolholl-2024).
  - `res.csv` is generated *from the `EffectLog` audit trail* — the residual
    history is a typed provenance record of the solve, not a printf. One README
    sentence: every number in the output files is traceable to a logged chain
    step.

**D4 — Composition as the coupling mechanism** (his #6 — "propagation of
uncertainties from solver to solver" unresolved, no standard exchange formats).
Demonstrated in miniature: the scored run, the UQ mode, and the extraction stage
are *recompositions of one typed chain*, not code forks — same march arrow, same
projection bind, different stage wiring; the UQ mode literally is uncertainty
propagating stage-to-stage through typed channels. The README states the
observable fact: three capabilities, one chain, zero `#ifdef`-style forking.

Deliberately *not* demonstrated: HPC (his #1 — single-core rule anyway, and our
stance is documented elsewhere), turbulence closures (his #2 — Re 1000 laminar
cavity needs none; do not pretend otherwise), and grid (his #4 — the scored run
mandates a uniform mesh; the grid program in `../cfd/variable-grid-geometry.md`
is follow-up-conversation material, one closing link at most).

## 3. Why Rust, as the README will state it (two sentences, no more)

C++ is the field's default because the incumbent solvers are C++ and education
bends to industry; the language is not load-bearing for the mathematics. The
composition machinery this solver rides (typed effect channels, higher-kinded
abstractions across modules) is theoretically replicable in C++ and practically
not — and Rust's falling certification cost in regulated industries is the trend
this solver's audit-trail design anticipates.

## 4. Mechanics: `evaluate.py` runs unmodified

His evaluator needs `CMakeLists.txt`, parses the `add_executable(` line for the
binary name, configures Release/Ninja, builds, runs `build/<name>`, times total
execution (I/O included — keep output writing lean). The entry ships a CMake shim
that (1) drives `cargo build --release --locked` at build time, (2) places the
binary at `build/<name>`, (3) carries a literal `add_executable(<name> ...)` line
the parser resolves. Dependencies vendored (`cargo vendor` + checked-in cargo
config) so the build is hermetic; the only environmental requirement is a Rust
toolchain — one `rustup` line, stated politely and early in the README. Risk
accepted and pre-empted in the README: if the language deviation excludes the
entry from the leaderboard, the comparison data is his to use anyway.

## 5. Required artifacts

- `README.md` — his exact template, answered honestly (co-written with Claude;
  the challenge *wants* LLM-authored code — language is the deviation,
  authorship is compliant). Solver description: MAC grid, projection, CG,
  convergence criterion, plus one sentence per D1–D4. The two §3 sentences.
  Links: repo + the CFD note deck. Nothing else.
- `chat_history.json` — genuine and curated for sensitive content only. He has
  committed to analyzing it; the prompts should *exhibit* the
  verification-first, structure-preserving reasoning (D1–D4 being designed,
  debated, validated) rather than announce it.
  - **Scope boundary (settled 2026-06-11):** the chat history covers the
    *submission unit* — the solver crate (numerics assembly, chain wiring, CSV
    writers, CMake shim). Library gap-closure work is upstream dependency
    development, exactly as Eigen/PETSc history would be for a C++ entry, and the
    challenge explicitly permits unlimited external dependencies. To make the
    boundary factual rather than rhetorical: **close the gaps first, release them
    as versioned crates.io updates, then author the solver against the pinned,
    vendored, published versions.**
  - **Disclosure, one sentence in the README:** the solver calls into the
    author's own published open-source libraries (LF-governed; developed
    spec-first with LLM assistance; full history public in the repository and its
    `openspec/` folder); this chat history covers the solver itself. For his
    research question this is added data, not a caveat — two LLM development
    modalities (conversational solver generation + long-horizon spec-driven
    library development), both inspectable.
- `uy.csv`, `vx.csv`, `res.csv` — exact spec formats. Auxiliary outputs
  (`div.csv` or a max-div column, vortex table, UQ bands) in clearly separate
  files so the required three stay pristine.
- CMake shim + vendored workspace.

## 6. Timeline (deadline 2026-07-07)

| Week | Deliverable |
|---|---|
| 1 (by 06-18) | Scored core: MAC + projection + CG converging; centerlines eyeball-correct vs. Ghia; profile the Poisson solve |
| 2 (by 06-25) | D1–D4: max-div reporting, UQ mode, vortex-table extraction, chain refactor; accuracy tuned to low-single-% RMSE |
| 3 (by 07-02) | CMake shim + vendoring verified against his `evaluate.py` on a clean machine; README + chat history |
| Buffer (07-03 → 07-07) | Slack; submit early, not at the wire |

Scope guards: D2 is N re-runs of a seconds-fast solver (Monte Carlo at the chain
level), not `Uncertain<R>` threaded through field arithmetic — that would blow
the budget for no visible gain. D3's extraction is the shipped Q-criterion kernel
plus small labelling code. If week 2 runs hot, D4 is free (it is the architecture)
and D2 drops to a smaller sample count before anything else gets cut.

## 7. What this entry is, in one line

A competitive, rule-respecting cavity solver whose output files quietly do four
things its judge has publicly stated the field cannot do — so that the
conversation about the solver category behind it begins on his initiative.
