[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# DSL Acceptance: What Must Be True

STATUS: assessment, feeding the eventual `cfd-study-grammar` change as its acceptance
criteria. Companion to `02-dsl-study-io.md` (the CfdFlow language rework, revision
4.1).

Two questions, answered separately and then joined: what must be true for the language
to become real, and what must be true for a CFD engineer to embrace it. They are the
same requirement seen from two sides, and the closing section makes the join explicit.

## 1. Engineering Reality Conditions

### 1.1 The type system is proven to compose before the spec freezes

STATUS: PASS. The compile spike ran before the spec (see `04-dsl-feasibility.md`). All
three novel constructs compose against the real generic arities; the three forbidden
programs are rejected by the type system. The spike forced three refinements (F1
`GateSeq<Row>` with HRTB gate fns, not a `'static` view; F2 `Marchable` spans the
uncoupled families plus the `Coupled<C, S>` wrapper; F3 `ForkStudy` threads the pause's
`'c, R, S, M`), all folded back into `02-dsl-study-io.md`. The record below states the
gate the spike satisfied.

The substrate itself carries no risk: the phase-chain-inside-one-effect construction
(plain typestate structs wrapped in an effect carrier whose haft HKT witness supplies
`Functor`/`Applicative`/`Monad`, double-impl fluent verbs, witness types hidden behind
the facade) is exactly the Causal Discovery Language's architecture, compiled, tested,
and shipped in `deep_causality_discovery`. The study grammar inherits a proven
pattern, including the precedent of compile-time-isolated sub-pipelines converging on
one tail (SURD/BRCD into `WithAnalysis`, as the three case binders into `Swept`).

Three constructs remain genuinely novel and load-bearing:

- `Marchable` unifying five config families whose generics differ in shape
  (`MarchConfig<D, R, Z, C>` against `DuctConfig<R>`); it needs a GAT pipeline
  associated type and careful bound design.
- The study phases carrying a borrow of a paused trajectory (`ForkStudy<'p, T>`)
  through a data-parallel sweep under `scoped_map`'s `Send + Sync` bounds.
- `GateSeq` over a view type, which needs higher-ranked `fn` signatures rather than
  the `'static` placeholder in the design sketch.

Each is solvable; none is allowed to be assumed. The proposal is validated by a
compile spike on a throwaway branch where these three compile against the real
`CompressiblePause`, before any OpenSpec delta is written. If one of the three does
not compose, the grammar quietly degrades back into closures, which is the failure
the redesign exists to prevent.

### 1.2 `MarchState` is real, not aspirational

The one-state-two-transports promise (pause it, resume it next line or next week,
bit-identically) is the strongest sentence in the design and currently three-quarters
true: `pack_resume` already captures fields, scalars, navigation engine, provenance
log, and step for the disk transport. The remaining quarter is an owned in-memory
state that `run_until` accepts and that equals the disk round trip bit-for-bit at
every scalar including `Float106`. Its test is exact: pause, save, load, continue
versus pause, continue; the two reports must be identical to the last bit.

### 1.3 Every guarantee becomes a test; every forbidden program becomes a compile-fail

The design states its semantics as guarantees. Each lands twice:

- Behavioral tests: order preservation and first-error-wins on the sweep, alternation
  marker presence on both counterfactual forms, fork bit-identity up to the pause,
  `read(write(t)) == t` per scalar, checksum rejection with `force_load` override,
  warning accumulation across the effect (a `force_load` override's warnings must
  reach the `Verdict`), rounds retention under `refine`, `Verdict::merge`
  completeness, and the audit log's three properties: disk-equals-memory (each
  completed `save_log` file renders identically to its thread's in-memory effect log,
  the main file closed by the verdict summary), one-thread-one-file (a forked run
  produces exactly one file per branch, exclusively written, named by round and case;
  the main file names every spawn and every rejoin outcome), and the per-file
  abort-tail (a branch killed mid-march leaves its own file ending at its last
  recorded event while the main file names the casualty; tested with a child
  process).
- Compile-fail doctests (` ```compile_fail ` blocks, no external test crates) on the
  phase types, one per entry in the design's "what no longer compiles" list.

This is what "works exactly as the syntax makes the user expect" means operationally:
the expectation is written down and enforced twice, once by rustc, once by the suite.

### 1.4 The migration is the proof of completeness

All seven programs (nozzle, VIV, placard, corridor, weather, the stagnation-line
study, `compressible_carrier_timing`) land on the grammar in the same change with
identical measured outputs: the corridor's gate set, the weather table's six gates,
the same shock stations and Strouhal numbers as the shipped `output.txt` records. The
corridor is the acid test; if it needs one escape hatch the grammar does not provide,
that gap goes back into the design, never into a workaround. Wall-clock stays inside
the existing gate budgets, which verifies the zero-cost claim instead of asserting it.

### 1.5 Compiler errors speak the domain

The characteristic failure of a typestate DSL is rustc naming
`Judged<PlacardRow, Shock>` at a confused engineer. The mitigation is mundane and
mandatory: `#[must_use]` on every phase, doc comments on every verb written as the fix
("there are no rows to record yet; `record` comes after `march` or `sweep`"), and a
review pass in which each forbidden program's actual error message is read and judged
as documentation.

## 2. Engineer Adoption Conditions

### 2.1 Trust arrives through V&V, never through syntax

An engineer adopts a tool whose numbers survive a design review. The dedicated
verification page with the deviation table (cylinder Strouhal and drag against the
Williamson reference, MMS observed orders, the RAM-C II anchors), every row
reproducible by one `cargo run`, is the entry ticket. The gate-and-verdict culture is
then the differentiator: their own studies inherit the self-verification habit.

### 2.2 The vocabulary is their vocabulary; the types stay invisible until violated

Test matrix, case, sweep, data reduction, record, gate: wind-tunnel and flight-test
language. The documentation presents the grammar as their existing workflow
formalized, never as type theory. The engineer writes the happy path by copying a
cookbook page; the phase types surface only as a helpful refusal.

### 2.3 Minutes to first result

Clone, run the nozzle, watch five gates pass, edit one number in
`back_pressures.csv`, rerun. If first contact requires understanding ownership or any
category theory, they are gone. The examples are the onboarding, which is why the
example structure convention and the READMEs carry adoption weight equal to the
library code.

### 2.4 Escape hatches that keep them in the ecosystem

The next study will exceed the grammar somewhere. The two-level design plus
`Verdict::merge` must make dropping to the trajectory level or to plain Rust feel
like using the language, not leaving it. A grammar experienced as a cage gets
abandoned; experienced as the short path, it gets defended.

### 2.5 Their data and their eyes

Typed CSV covers scalar studies. The day an engineer wants to see the committed
branch's flow field in ParaView is the day the tool either crosses from demo to
instrument or does not; field export (VTK/CGNS, roadmap item 5) is the known boundary
and is stated as such rather than papered over.

### 2.6 Stated boundaries and surface stability

The release criterion already set for the crate applies verbatim: a toolbox that
works for a stated category of problems, documented within clearly stated boundaries,
explicit about today versus roadmap. The corollary for the DSL: the rework lands
completely while `deep_causality_cfd` is still `publish = false`. Today the entire
surface can be rebuilt with zero external breakage; after first publication every
verb is a commitment. The window for a no-compromise rework is exactly now, which is
the strongest argument for doing it whole rather than incrementally.

## 3. The Join: One Chain Per Verb

The engineer's trust criteria and the language's semantic guarantees are the same
list. Reproducibility is bit-identical forks and checksummed resume. Auditability is
the alternation marker and the provenance log. Defensibility is the gate sequence.
Exactness is precision as a parameter through the codec.

Therefore the unit of doneness is per verb, and it is a chain of four:

1. a **stated guarantee** in the spec,
2. the **test** that enforces it (behavioral or compile-fail),
3. the **shipped example** that demonstrates it,
4. the **doc page** that names it.

A verb missing any link is not done. This chain is the acceptance criterion the
`cfd-study-grammar` change inherits.
