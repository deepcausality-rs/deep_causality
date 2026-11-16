# Consistency and organization plan

This plan brings DeepCausality's public text and example layout into line with what the code does. It
comes before any paper or monograph rewrite. The core framing does not change.

**Reference.** The code is the primary source. The root `README.md` is the reference for how the
project describes itself. It opens with the framing every other surface should follow:

> Dynamic causal reasoning: cause and effect in systems whose state, environment and rules change
> dynamically.

**Checked for this plan.** The root `README.md`, `examples/README.md`, the `website/web` home page,
overview pages and examples index, the crates' opening docs, and the Cargo example targets. Line numbers
are from 2026-11-06. The `website/docs` site, the blog, the tutorials and the `cfd` and `quantum` sites
were searched for specific terms but not read, so item A8 covers them.

---

## Part A. Consistency

### A1. Use one description everywhere

Six wordings are in use: "dynamic causal reasoning", "dynamic causality", "computational causality",
"causal-reasoning library", "framework" and "library".

| File | Line | Current text | Action |
|---|---|---|---|
| `website/web/src/pages/index.astro` | 47-48 | "Computational Causality Framework in Rust", "framework for computational causality" | Align the title and meta description |
| `website/web/src/pages/index.astro` | 24, 40 | "framework for dynamic causality" | Align the structured data |
| `website/web/src/components/home/Hero.astro` | 12-13 | "causal-reasoning library for engineering systems", "one carrier effect propagating between them" | Use the README sentence and drop the jargon |
| `website/web/src/pages/about.astro` | 18 | "dynamic-causality framework" | Align, see also A6 |
| `deep_causality/src/lib.rs` | 6 | "Rust framework for dynamic causality" | Align |
| `deep_causality/Cargo.toml` | 14 | "Computational causality library" | Align, because crates.io shows this text |
| `deep_causality_core/Cargo.toml` | 11 | "Core types for deep_causality crate." | Describe the causal monad |
| `AGENTS.md` | 172, 212 | "computational causality library" | Align |

- [ ] **A1.1** Decide between "library" and "framework". The README says library, so use library unless
  there is a reason not to.
- [ ] **A1.2** Apply the table.
- [ ] **A1.3** Search `website/docs`, `website/cfd` and `website/quantum` for the same variants and align
  them.

**Done when** a search for "computational causality" and "causal-reasoning library" finds only
historical blog posts, and the algorithm crate description where the phrase names its subject.

### A2. Make the code keep what the text promises

These are code fixes. `papers/src/rework-epp.txt` sections 3.0 and 3.1 hold the detail.

- [ ] **A2.1** Evaluate Collection and Graph causaloids through `Causaloid::evaluate` and
  `evaluate_stateful`, at any depth. Today both return `UnsupportedOperation`.
- [ ] **A2.2** Replace the tests that hide this failure. Section 3.0 lists them.
- [ ] **A2.3** Rule on how state behaves at a join in the stateful graph engine.
- [ ] **A2.4** Fix `flight_envelope_monitor`. Its Nominal case reports the join error, while its README
  expects `verdict=Nominal`.
- [ ] **A2.5** Design and add the do operator over the causal monad and the causaloid. The design is
  open.

### A3. Correct the text that depends on A2

The README and the dynamic causality page already say that `evaluate` runs a singleton and that the
stateful engine errors at a join. These passages conflict with those statements.

| File | Line | Current text | Interim action | After A2 |
|---|---|---|---|---|
| `website/web/src/components/home/WhyDeepCausality.astro` | 22-23 | "hypergraph of nested Causaloids", "deeply composed causal structure with subgraphs" | Remove nesting | Restore with a working example |
| `deep_causality/src/lib.rs` | 12 | "recursive causal data structures" | State that the type is recursive and evaluation is per form | Restore |
| `README.md` | 182 | Offers `flight_envelope_monitor` as a larger example | Replace with a working example | Restore |
| `README.md` | 369 | The monitor "uses all three" shapes | Note the current failure or link a fix | Keep |
| `website/web/src/pages/overview/index.astro` | 70 | The monitor "runs a collection ... and a graph ... in one process" | Same | Keep |
| `website/web/src/pages/overview/dynamic-causality.astro` | 256-262 | Says the engine errors at a join, then presents the monitor | Resolve the contradiction | Update both statements |
| `README.md` | 351-352 | `Causaloid::evaluate` evaluates a singleton | Keep, it is accurate | Update |
| `README.md`, `overview/dynamic-causality.astro` | 265, 90 | "no step can feed back into an earlier one" | Add that `RelayTo` can jump to any node, bounded by a relay limit | Keep |

- [ ] **A3.1** Apply the interim actions now.
- [ ] **A3.2** Apply the "After A2" column when A2 lands.
- [ ] **A3.3** Correct `deep_causality/LEAN_DeepCausality.md` and `lean/THEOREM_MAP.md`, which describe
  `evaluate` as the catamorphism. That holds for the Lean model until A2.1 lands. Rewrite the encapsulation
  witness so it nests.

### A4. Say one thing about `do`

The sources disagree about whether `alternate_value` is `do`.

| File | Line | Says |
|---|---|---|
| `README.md` | 158 | Replacing the value "is Pearl's intervention" |
| `examples/README.md` | 13, 103 | "Pearl-style one-shot `do`-operator interventions" |
| `examples/README.md` | 109, 113 | `do(T=1)`, `do(connectome = resected_at_R)` |
| `deep_causality_core/README.md` | 128 | Value substitution, with `do` in the causaloid layer |
| `deep_causality_core/src/types/causal_flow/intervene.rs` | 10-14 | These are not `do`, which lives in the causaloid layer |

The causaloid layer has no such operator.

- [ ] **A4.1** Agree one interim sentence and use it in every row. Suggested: "`alternate_value` replaces
  a value mid-run. The do operator over the monad and the causaloid is planned."
- [ ] **A4.2** Replace `do(...)` in the example tables with what each example does, until A2.5 lands.
- [ ] **A4.3** Review the do wording in `website/docs`: `concepts/counterfactuals.md`,
  `concepts/causal-flow.md`, `concepts/dynamic-causality.md` and `overview/innovations.md`.
- [ ] **A4.4** Revisit all of these when the do design is settled.

### A5. Keep counted facts in one place

| Fact | `README.md:456`, `overview/index.astro:116` | `AGENTS.md` |
|---|---|---|
| Library crates | 33 | 34 |
| Crates with no external runtime dependency | 26 | 27 |

- [ ] **A5.1** Confirm the counts from the workspace with `scripts/crates.sh`.
- [ ] **A5.2** Correct both pages.
- [ ] **A5.3** Derive the numbers in a check script, or state them without a count.

### A6. Bring the older pages up to the README

- [ ] **A6.1** `website/web/src/pages/about.astro:18-23`. Replace the opening with the README sentence.
  Verify the statement that the library is used in IoT, cloud and finance, or remove it.
- [ ] **A6.2** `website/web/src/pages/examples/medicine.astro:12`. Describe what the examples do for a
  patient or clinician, not "a Metropolis-style optimizer wrapped in a PropagatingEffect".
- [ ] **A6.3** Check the descriptions on the aerospace, physics, mathematics and foundations pages the
  same way.
- [ ] **A6.4** `deep_causality/src/lib.rs:6-12`. Review the "Why DeepCausality?" list against the README.

### A7. Keep "How it works" about how it works

`README.md:247` opens the section with Whitehead's process metaphysics and one axiom. Everything before
it describes what the project does, and the two-layer explanation after it works without that opening.

- [ ] **A7.1** Author decision: move the philosophical origin to a short linked note, or keep it.
- [ ] **A7.2** Apply the same decision to `website/docs/src/content/docs/concepts/axiom.md`.

### A8. Audit the surfaces not yet read

- [ ] **A8.1** Read the 44 files under `website/docs/src/content` against the code. Start with the files
  that mention nesting or subgraphs: `getting-started/hello-causaloid.md`, `formalization/core.md`,
  `overview/architecture.md`, `concepts/dynamic-causality.md`, `getting-started/hello-causal-flow.md`,
  `concepts/axiom.md`.
- [ ] **A8.2** Read the tutorials and the example detail pages.
- [ ] **A8.3** Fix the stale material in `papers/src/rework-epp.txt` section 3.7: broken blog links, removed
  API names, and the README disagreements listed there.
- [ ] **A8.4** Decide how dated blog posts are handled. Suggested: keep them, and add a dated note where a
  post describes an API or claim that no longer holds.

### A9. Mark the outdated monograph

- [ ] **A9.1** Add a status note wherever the monograph or `epp.pdf` is offered, saying it predates the
  current code and is being rewritten. Two blog posts link it: `announcement-effect-ethos.md` and
  `announcement-adaptive-reasoning.md`.
- [ ] **A9.2** Point readers to the README and docs as current.

---

## Part B. Organization

### B1. State the point

The examples exist because dynamic causal processes are everywhere. A search of the README, overview and
home pages does not find that statement, and the site currently says "One framework, many fields" and
"Pick the field that looks closest to the problem".

- [ ] **B1.1** Author: settle the wording. A draft built from the README: "A dynamic causal process is one
  whose state, environment or rules change while it runs. These processes occur in every field, so the
  examples come from many fields and show the same few operations."
- [ ] **B1.2** Place it in `README.md` before "Run larger examples", at the top of `examples/README.md`,
  on `website/web/src/pages/examples/index.astro:17-18`, and above the home page example grid.

### B2. Organize by process first and field second

`examples/README.md` mixes two axes: operations (counterfactual, correction, CSM) and fields (physics,
medicine, quantum). Readers then judge the depth of one field. The correction and counterfactual groups
already show the better form: one process across several fields.

Change the index first and leave the folders alone, so package names, Bazel labels and links stay valid.

Starting sort, from index descriptions only. Confirm each entry by reading the example.

| Process | What happens | Examples to start from |
|---|---|---|
| Step by step | State carries forward through a chain of mechanisms | `gentamicin_dosing`, `bernoulli_flow_network`, `sensor_processing`, `clinical_trial`, `multi_physics_pipeline` |
| The law changes | The state selects which mechanism applies | `event_horizon_probe`, `grmhd_example` |
| What if | A value, state or world is replaced, and the run continues | the six `counterfactual_*`, `quantum_counterfactual`, the Pearl ladder examples (`scm_*`), `plasma_blackout_weather` |
| Correct and continue | A monitor detects drift, intervenes, and the run continues | the five `corrective_*`, `geometric_tcas` |
| Sense, decide, act | A result triggers an action, optionally checked against norms | `csm_*`, `csm_effect_ethos_example` |
| Combine causes | Several causes join in a collection or graph | `flight_envelope_monitor`, `starter_example`, `dbn_*`, `rcm_*` |
| Find causes in data | Structure and root causes are discovered from data | the seven `causal_discovery_examples` |
| Mathematics underneath | The shared mathematical layer | the 77 `mathematics_examples` |

- [ ] **B2.1** Author: approve or change this list of processes. Keep it short.
- [ ] **B2.2** Sort every example, reading each one. An example that fits no process goes to B4 or gets
  a note, not a forced fit.
- [ ] **B2.3** Rewrite `examples/README.md` around the processes, with the field shown in each row.
- [ ] **B2.4** Keep a field view as a second table, so a reader can still search by field.

### B3. Change the site the same way

- [ ] **B3.1** `website/web/src/pages/examples/index.astro`. Lead with the processes and offer field as a
  filter.
- [ ] **B3.2** Reorganize `website/web/src/content/examples/en` categories to match. Keep existing URLs or
  add redirects.
- [ ] **B3.3** Check that each process has one detail page with a real example.
- [ ] **B3.4** Review the home page tabs in `components/home/examples.ts` against the same list.

### B4. Separate the foundation from the processes

71 of the 77 mathematics examples show no carrier or causaloid by text match. Many demonstrate the
mathematical layer, and that is their purpose.

- [ ] **B4.1** List `1_foundation` and most of `2_composition` as "the mathematics underneath", linked from
  the processes that use them.
- [ ] **B4.2** Move a `3_applications` example or `diffusion_space_and_time` into a process group where it
  shows one.

### B5. Open every example the same way

- [ ] **B5.1** Add a short block to each example README, using the terms from B1:
  - what propagates from step to step
  - what state it carries
  - what context it runs in
  - what changes along the way
  - what it covers and what it leaves out
- [ ] **B5.2** Apply it first to the examples named in B2, then to the rest.
- [ ] **B5.3** Add the block to the template rules in `AGENTS.md`, so new examples follow it.

The scope line deals with the "nothing full" impression, because a stated limit reads as a choice. The
flight envelope README already does this.

### B6. Show more of the operations

`papers/src/rework-epp.txt` section 3.8 holds the counts and detail.

- [ ] **B6.1** Add a what-if to several physics examples. None of the 13 intervenes by text match.
- [ ] **B6.2** Add examples for `RelayTo`, `Model::evolve`, unfreeze and refreeze, and `verify_assumptions`.
  None appears in an example today.
- [ ] **B6.3** Add a nested causaloid example after A2.1.
- [ ] **B6.4** Add rung 3 and do examples after A2.5.
- [ ] **B6.5** Author decision: which example leads each process, including whether the re-entry family
  stays on the README run list. The open export question in the project notes bears on this.

### B7. Fix the index errors

Checked by comparing the commands in `examples/README.md` with `cargo metadata`.

- [ ] **B7.1** `starter_example` is a binary, so `--example starter_example` fails. This is the entry marked
  "Start here if you are new". Correct the command.
- [ ] **B7.2** `tokio_example` has the same problem.
- [ ] **B7.3** `physics_examples --example algebraic_scanner` is not registered. Remove the row or correct
  it. The mathematics group has `algebraic_scanner_examples`.
- [ ] **B7.4** `csm_battery_example` is registered and missing from the top index, and 10 of the 16 quantum
  examples are too. Add them, or state that the index lists highlights, as mathematics does.

---

## Part C. Keeping it consistent

- [ ] **C1** Add an index check to `make check_examples`: every command in the README files must resolve to
  a registered target, and every target must appear in an index.
- [ ] **C2** Add an example smoke test to CI. An unexpected error must fail it. `flight_envelope_monitor`
  shows the gap: its header documents exit status zero for both scenarios, including the failing one, so
  the example reports success while its Nominal case errors. Give each example a stated expected outcome.
- [ ] **C3** Add a link check over `website/web`, `website/docs` and the READMEs.
- [ ] **C4** Derive counted facts in a script, as in A5.3.
- [ ] **C5** Add one rule to `AGENTS.md`: a public claim about a capability names the code or example that
  shows it. If none exists, the text says it is planned.

---

## Order of work

| Step | Items | Why here |
|---|---|---|
| 1 | B7, A5, A1 | Small and visible, with no design needed |
| 2 | A3.1, A4.1, A4.2, A9 | Stops the text promising what the code does not do |
| 3 | B1, B2 | Your point stated, and the structure everything else follows |
| 4 | A6, A7, A8 | Older pages and unread surfaces, against the settled wording |
| 5 | B3, B4, B5 | Applies the structure to the site and the examples |
| 6 | A2.1 to A2.4, A3.2, A3.3, C2 | Code fixes, with the text restored as each lands |
| 7 | A2.5, A4.4, B6 | Do design, then the examples that show it |
| 8 | C1, C3 to C5 | Guards, added as each area settles, and earlier where cheap |

## Decisions for the author

| Item | Decision |
|---|---|
| A1.1 | Library or framework |
| A2.3 | State at a join in the stateful graph engine |
| A2.5 | The do design |
| A4.1 | The interim sentence on `do` |
| A6.1 | The IoT, cloud and finance statement on the about page |
| A7.1 | Where the philosophical origin goes |
| A8.4 | How dated blog posts are handled |
| B1.1 | The wording of the central point |
| B2.1 | The list of processes |
| B6.5 | Which example leads each process, including the re-entry family |
