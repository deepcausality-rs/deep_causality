# deep_causality_cfd — documentation-vs-code parity (crate README, lib.rs rustdoc, verification/ and studies/ READMEs, Cargo.toml, reverted/, benches/PERFORMANCE.md, module rustdoc)

**Production readiness: `needs-work`**

Item-level rustdoc is genuinely complete: every one of the ~200 names re-exported from `src/lib.rs` resolves to a definition with a preceding `///` block, and the substantive DSL claims I could mechanically check hold up — the O(1) copy-on-write fork is `Arc::clone` with measured (not asserted) `ForkEconomics` at carrier.rs:625-631, concurrent-vs-sequential bit-identity is a real test at fork_tests.rs:71-81, `verdict()` returns data with no print or exit (reduce.rs:171-175), `NAV_STATES = 17` (eskf.rs:25), and the Couette/Poiseuille validations the README cites do exist as runnable tests. Against that, the shipped prose fails an avionics traceability bar in several concrete ways. `verification/README.md` twice tells a reader to drive the QTT solvers through `CfdFlow::qtt_march`, a method that does not exist anywhere in the workspace. The crate README's headline precision result ("at 106-bit Float106 every gate is identical") exists only as prose, and its own source — `corridor/README.md:199` — labels it as measured on a superseded "surrogate-era build". `parallel` is a *default* feature in Cargo.toml:38 while `src/lib.rs:22` and the Cargo.toml comment itself both say it is opt-in and serial by default, and `benches/PERFORMANCE.md` measures that default configuration as 1.4×–3.2× *slower* below 256². Traceability artifacts are thin: `papers/` holds four PDFs against roughly twenty cited works, and five of thirteen verification programs — including the RAM-C II flight-data comparison — have no per-example section and no entry in the References list that the same README promises. Finally, an entire powered-descent/retropulsion subsystem, the duct marcher, the snapshot/resume surface, and the `AcousticCoreInverse` family are public and undocumented outside their own source files. None of these are wrong physics; they are reproducibility and traceability defects, and they are all cheap to fix.

- Files read: **52**
- Findings raised: **16** — surviving adversarial verification: **16** (refuted: 0)
- Surviving by severity: major 5, minor 9, info 2
- Independently confirmed-correct items: **11**

## Verified correct against reference

These were positively confirmed, not merely un-flagged.

| Item | Location | Reference checked against |
|---|---|---|
| README claim: "verification/ holds thirteen runnable programs" | `deep_causality_cfd/README.md:214; Cargo.toml:138-190` | Count of [[example]] stanzas whose path begins with verification/, cross-checked against `ls verification/` |
| README claim: "A fork shares the paused state in O(1) through copy-on-write" | `deep_causality_cfd/src/types/flow/carrier.rs:625-631` | O(1) sharing = reference-count bump, not a deep copy of the tensor state |
| README claim: "branch fan-outs run concurrently on scoped threads and produce bits identical to the sequential run" | `deep_causality_cfd/tests/types/flow/compressible_march_run/fork_tests.rs:71-81; deep_causality_par/src/functions/scoped_map.rs:61-69` | Parallel result == result of the equivalent sequential fork chain, compared by value equality on the final field and series |
| README claim: "the DSL never exits or prints (verdict() returns data)" | `deep_causality_cfd/src/types/flow/study/reduce.rs:171-175` | Terminal combinator returns Result<Verdict, StudyError> with no side effect on stdout/stderr and no process::exit |
| Every Coupling::between_steps() identifier in the README snippet exists with the stated arity | `coupling.rs:344,351,358; blackout.rs:163; finite_rate_ionization.rs:68,101; corridor/regime.rs:174; corridor/branch.rs:123; corridor/trajectory_nav.rs:51,62; corridor/gate.rs:44; corridor/envelope.rs:82` | Each named constructor/builder method in README.md:161-168 must exist with a signature accepting the arguments shown |
| README table claim: src/navigation/ holds "the 17-state error-state Kalman engine" | `deep_causality_cfd/src/navigation/eskf.rs:25` | Stated state dimension must equal the declared constant |
| README claim: validation against "exact Couette and Poiseuille states" | `deep_causality_cfd/tests/solvers/dec/poiseuille_tests.rs:139-145; tests/solvers/dec/dec_ns_solver/no_slip_tests.rs:305-317; tests/theories/stokes_verification_tests.rs:43,84,114` | Runnable checks against the analytic plane-Couette linear profile and plane/Hagen-Poiseuille parabolic profile (Batchelor 1967 §4.2, Eq. 17.4) |
| Complete rustdoc coverage of the lib.rs public surface | `deep_causality_cfd/src/lib.rs:37-155` | Every name in a `pub use` list resolves to a definition preceded by a /// doc comment |
| Precision policy in the MMS reference path: exact f64 specification lifted, not computed, at working precision | `deep_causality_cfd/src/types/flow_config/manufactured.rs:108-113, 199-202` | README.md:238-239 "Specification constants stay exact f64 literals; ft lifts each one into the working precision, and every derived number is computed in FloatType" |
| README's quoted regime-transition log lines are producible by the current format string | `deep_causality_cfd/src/types/flow/corridor/regime.rs:346-353; corridor/output.txt:56-67` | README.md:147-150 quoted text must match the format!("regime -> {} ({}), Kn={}{}") output |
| README claim: the corridor commits "the best of the seventeen worlds (six coarse + eleven fine)" | `examples/avionics_examples/cfd/plasma_blackout/corridor/model.rs:47-53, 74-87` | len(coarse_commands()) + len(fine_candidates()) == 17 |

## Findings

### 15.1 [MAJOR] verification/README.md instructs readers to use CfdFlow::qtt_march, a method that does not exist

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/verification/README.md:192`
- **Auditor confidence:** confirmed

**Claim.** The verification README and two per-example READMEs document the QTT solvers as being "Driven through `CfdFlow::qtt_march`". No such associated function exists anywhere in the workspace; the actual entry point is `CfdFlow::march`. Any reader following the documentation gets a compile error.

**Code evidence.**

```
verification/README.md:192 "compression (bond vs. dense) is reported. Driven through `CfdFlow::qtt_march`."
verification/README.md:213 "analysis (immersed body + surface observables). Driven through `CfdFlow::qtt_march`."
verification/qtt_taylor_green_verification/README.md:7 and :87 repeat it; verification/qtt_cylinder_verification/README.md:6 and :71 repeat it.

But `grep -rn "fn qtt_march" --include=*.rs .` over the whole workspace returns nothing (exit 1).

The only public methods on CfdFlow are:
  src/types/flow/cfd_flow.rs:103  pub fn study(title: &str) -> StudyDef
  src/types/flow/cfd_flow.rs:115  pub fn march<R, Cfg>(config: &Cfg) -> Cfg::Pipeline<'_>
  src/types/flow/verify.rs:22     pub fn verify<R, M>(config: &VerifyConfig<R, M>)
  src/types/flow/operator_study.rs:33 (operator study entry)
  src/types/flow/mms.rs:48 (mms entry)

And the harnesses those READMEs describe actually call `march`:
  verification/qtt_taylor_green_verification/main.rs:77  let report = CfdFlow::march(&case_config)
  verification/qtt_cylinder_verification/main.rs:61      let report = CfdFlow::march(&case)
```

**Reference form.** The crate's own public API: `CfdFlow::march(&config)` (src/types/flow/cfd_flow.rs:115), which is what the harnesses in the same directories call.

**Impact.** An engineer reading the verification documentation to reproduce or extend a QTT check writes `CfdFlow::qtt_march(...)` and hits E0599. More seriously for a pre-certification review, it signals that the verification README has drifted from the code it documents and was not re-validated against it — which undermines confidence in the measured numbers in the same file.

**Recommended fix.** Replace all five occurrences of `CfdFlow::qtt_march` with `CfdFlow::march` in verification/README.md:192,213, verification/qtt_taylor_green_verification/README.md:7,87 and verification/qtt_cylinder_verification/README.md:6,71. Add a CI doc-check that greps shipped Markdown for `CfdFlow::<ident>` and fails on any identifier absent from the CfdFlow impl blocks.

**Adversarial check.** `grep -rn "fn qtt_march" --include=*.rs .` over the whole workspace returns nothing. The only `pub fn` on CfdFlow in cfd_flow.rs are `study` (103) and `march` (115). All six quoted doc occurrences of `CfdFlow::qtt_march` exist verbatim at the cited lines, and both harnesses in those same directories call `CfdFlow::march`. `qtt_march` survives only as a config *name* string (qtt_march_config.rs:324) and module names, never as an associated function. Nothing compensates this — a reader following the doc gets E0599.

> Evidence re-read: deep_causality_cfd/src/types/flow/cfd_flow.rs:103,115 (only two pub fns); verification/README.md:192,213; qtt_taylor_green_verification/README.md:7,87; qtt_cylinder_verification/README.md:6,71; qtt_cylinder_verification/main.rs:61 `let report = CfdFlow::march(&case)`; qtt_taylor_green_verification/main.rs:77 `CfdFlow::march(&case_config)`

---

### 15.2 [MAJOR] The Float106 precision result is prose-only and its own source labels it as measured on a superseded build

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/README.md:240`
- **Auditor confidence:** confirmed

**Claim.** The crate README presents the corridor's 106-bit precision invariance as a current measurement of the shipping code. No artifact, gate, test, or committed output records it, and the example README that is its only source explicitly states the study was recorded on a superseded ("surrogate-era") build whose ionization kernel is no longer the one flown.

**Code evidence.**

```
README.md:240-244:
  "The plasma-blackout corridor measured all three: at `f64`,
   all gates pass in about 40 seconds; at 106-bit `Float106`, every gate and every discrete event
   step is identical, with continuous witnesses agreeing to 15 or 16 significant digits, which
   places the corridor's error budget in the model closures and the grid rather than in round-off;"

The sole source, examples/avionics_examples/cfd/plasma_blackout/corridor/README.md:198-206:
  "Three runs, same corridor (the precision study was recorded on the surrogate-era
   build; the network keeps the same SI-unit exponent ranges that set its conclusion):"
  | `f64` | All gates pass in about 35 s. The default. |
  | `Float106` (106-bit) | Every gate and every discrete event step identical; ... about 11x the wall-clock. |
  | `f32` | Crashes at step 1: `h^2` in the then-flown Saha kernel (4.4e-67) underflows ... |

The phrase "the then-flown Saha kernel" confirms the measured build used a different chemistry path than the current finite-rate network.

Searching for any recorded Float106 corridor artifact:
  grep -rn "Float106" over deep_causality_cfd/{src,verification,studies,tests} and examples/avionics_examples
  yields only (a) type-alias comments ("or f32, or deep_causality_num::Float106"),
  (b) one unrelated unit test (tests/solvers/dec/dec_ns_solver/step_tests.rs:89), and
  (c) the mms_taylor_green_verification README's own f32/f64/Float106 error table.
No corridor Float106 output.txt, no gate, no test.

The corridor is fixed at f64: corridor/main.rs:50 `pub type FloatType = f64;`
```

**Reference form.** The crate's own stated evidence standard, README.md:213 "The crate ships its evidence" and verification/README.md:13-17 "Every example self-verifies and exits with a nonzero status the moment its invariant or reference check fails — so the suite is usable as a gate, not just a demo." A precision-invariance claim meeting that standard would be a committed run artifact or a gate, not prose.

**Impact.** Precision invariance is exactly the kind of claim an avionics reviewer will lean on to bound the numerical error budget. Here it is unreproducible (no artifact), unverifiable (no gate), and — per its own source — measured against code that has since been replaced. The crate README strips the surrogate-era caveat that the example README carries, so the reader of the top-level document has no way to know the result is stale.

**Recommended fix.** Either (a) re-run the corridor at Float106 on the current finite-rate build, commit the output as a verification artifact, and cite it from README.md:240; or (b) demote the claim to prose that carries the same caveat as its source: "a surrogate-era precision study found every gate and discrete event step identical at Float106; it has not been re-measured against the current finite-rate network." Also reconcile the wall-clock: README.md:241 says "about 40 seconds" while corridor/README.md:204 says "about 35 s" and corridor/README.md:21 says "about 40 seconds".

**Adversarial check.** README.md:240-244 states the corridor 'measured all three' with the Float106 result presented unqualified. The sole source, corridor/README.md:198-206, reads verbatim: 'the precision study was recorded on the surrogate-era build' and 'h^2 in the *then-flown* Saha kernel', confirming the measured build is not the shipping one. I searched Float106 across cfd src/verification/studies/tests and examples/avionics_examples: every hit is a type-alias comment, a trait-bound remark (carrier.rs:471, duct_config.rs:97, cfd_scalar.rs:12), or the mms/turbulence_flow examples' own precision tables. The only executable Float106 code is examples/avionics_examples/cfd/turbulence_flow/main.rs:34 — an unrelated Lorenz study, not the corridor. corridor/main.rs:50 is fixed at `pub type FloatType = f64;`. No corridor artifact, gate, or test records the claim, against README.md:213 'The crate ships its evidence.'

> Evidence re-read: deep_causality_cfd/README.md:240-244; examples/avionics_examples/cfd/plasma_blackout/corridor/README.md:198-206; corridor/main.rs:50; exhaustive Float106 grep across cfd src/verification/studies/tests + examples/avionics_examples

---

### 15.3 [MAJOR] `parallel` is a default feature while lib.rs and Cargo.toml both document it as opt-in and serial-by-default

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/Cargo.toml:38`
- **Auditor confidence:** confirmed

**Claim.** The crate ships with `parallel` enabled by default, but three separate documentation sites state it is opt-in / off by default, and the crate's own benchmark document measures that default configuration as 1.4x-3.2x slower than serial for every grid below 256^2.

**Code evidence.**

```
Cargo.toml:38
  default = ["std", "parallel"]

Cargo.toml:51-53 (the comment directly above that same feature):
  # Opt-in CPU parallelism. Forwards to the topology crate's Rayon-backed DEC
  # operator loops and the shared `MaybeParallel` marker. Serial by default;
  parallel = [ ... ]

src/lib.rs:22-23 (crate rustdoc):
  //! CPU parallelism is opt-in via the `parallel` feature
  //! and rides the `MaybeParallel` bound.

benches/PERFORMANCE.md:31-34:
  > **Summary:** at the resolutions these benches use, the `parallel` feature does **not** help and
  > actively **hurts** the marching solver ... It is a knob for large grids, not the default.

benches/PERFORMANCE.md:44-47 (measured):
  |  16² |   1.680 ms |  5.410 ms | 0.31× (3.2× slower) |
  |  24² |   3.763 ms |  8.577 ms | 0.44× (2.3× slower) |
  |  32² |   7.060 ms | 12.215 ms | 0.58× (1.7× slower) |
  |  48² |  15.597 ms | 21.724 ms | 0.72× (1.4× slower) |

benches/PERFORMANCE.md:134-135 (guidance):
  - **Guidance:** keep `parallel` **off** below ~256² (small/CI-scale workloads);
```

**Reference form.** Cargo feature semantics: names listed in `default` are enabled unless the consumer passes `default-features = false`. "Opt-in" and "serial by default" describe a feature absent from the `default` list.

**Impact.** A consumer following the documented usage (`deep_causality_cfd = { git = ... }`, README.md:26-28, no feature overrides) silently gets the configuration the crate's own benchmarks say is up to 3.2x slower for typical grid sizes, while every doc site tells them parallelism is off. There is a secondary reproducibility concern: whether Rayon-backed reduction ordering in the DEC operator loops is bit-stable between the default and `--no-default-features` builds is not documented anywhere, so two engineers running the same case with different feature flags have no stated guarantee their numbers agree. (The scoped_map branch fan-out is deterministic; the topology operator loops are outside this audit's scope and I did not verify them.)

**Recommended fix.** Pick one and make all four sites agree. Given PERFORMANCE.md's own guidance, the consistent choice is `default = ["std"]`, leaving `parallel` genuinely opt-in; that also makes the Cargo.toml:51-53 comment and src/lib.rs:22 true as written. If `parallel` must stay in `default`, change lib.rs:22-23 to "CPU parallelism is on by default via the `parallel` feature; disable it with `default-features = false` below ~256² grids (see benches/PERFORMANCE.md)", fix the Cargo.toml comment, and add a note to README.md. Separately, document whether the two builds are bit-identical.

**Adversarial check.** Cargo.toml:38 is literally `default = ["std", "parallel"]`. The comment two lines above the feature itself (Cargo.toml:51-53) says 'Opt-in CPU parallelism ... Serial by default;' — the contradiction is inside a single file. src/lib.rs:22-23 repeats 'CPU parallelism is opt-in via the `parallel` feature'. benches/PERFORMANCE.md:31-34 says the feature 'does not help and actively hurts the marching solver ... It is a knob for large grids, not the default', and the measured table (16²/24²/32²/48² at 0.31x/0.44x/0.58x/0.72x) plus the guidance line 'keep `parallel` off below ~256²' all match verbatim. Cargo semantics are not in doubt: a name in `default` is on unless the consumer passes default-features = false. No compensating mechanism exists — nothing disables parallel at runtime below a grid threshold.

> Evidence re-read: deep_causality_cfd/Cargo.toml:38 and :51-53; src/lib.rs:22-23; benches/PERFORMANCE.md:31-34, :44-47, :134-135

---

### 15.4 [MINOR] Crate README frames the RAM-C II comparison as "validate", contradicting the verification README's own order-of-magnitude caveat

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/README.md:218`
- **Auditor confidence:** confirmed

**Claim.** The crate README omits the accuracy tier that the verification README attaches to this gate ('order-of-magnitude ... not a per-point accuracy claim', ±0.70-decade earned band), so a reader of the top-level document infers a quantitative-accuracy result. The measurement itself is correctly attributed to the plasma-blackout examples — the corridor gates against the RAM-C II anchor directly (corridor/README.md:20,65) in addition to verification/qtt_ramc_stagline.

**Code evidence.**

```
README.md:218-219:
  "The plasma-blackout examples validate an uncalibrated finite-rate ionization network
   against RAM-C II flight data."

verification/README.md:41 (the actual measurement):
  | `qtt_ramc_stagline` | peak electron density `n_e` / blackout onset | 1.085e19 (calibrated Park-2T);
    2.991e19 (uncalibrated network) | ~1e19 (RAM-C II, order-of-mag) | **+0.0 dec** calibrated;
    **+0.48 dec** prediction (earned band ±0.70) | stagnation line | ~1 s |

verification/README.md:45-49 (the caveat the crate README drops):
  "**Validation scope labels.** The QTT compressible gates verify at three distinct tiers — read each gate for
   what it actually proves: **analytic** (`qtt_sod` vs the exact Riemann solution — rigorous, the only
   quantitative-accuracy gate); **flight-data, order-of-magnitude** (`qtt_ramc_stagline` peak `n_e` vs RAM-C II;
   the Apollo re-entry dwell window is the corridor-time anchor, not a per-point accuracy claim)"

Note also that the crate README attributes the result to "the plasma-blackout examples", but the
measurement lives in verification/qtt_ramc_stagline, not in examples/avionics_examples/cfd/plasma_blackout/.
```

**Reference form.** verification/README.md:47-48, the crate's own scope label for this exact gate: "flight-data, order-of-magnitude ... not a per-point accuracy claim". A +/-0.70-decade band is a factor of 5.01 in each direction.

**Impact.** "Validate ... against flight data" is the strongest claim in the README, and it is the first thing an avionics reviewer will test. Read alone — which is how a top-level README is read — it implies a quantitative accuracy result. The supporting document says the opposite. A reviewer who discovers the gap themselves will discount the rest of the README's claims, including the ones I verified as correct.

**Recommended fix.** Restate README.md:218-219 in the verification README's own vocabulary, e.g.: "The `qtt_ramc_stagline` verification compares an uncalibrated finite-rate ionization network against RAM-C II flight data at order-of-magnitude scope: peak n_e lands +0.48 decades high inside a +/-0.70-decade band. This bounds the network's decade, not its per-point accuracy." Also correct the attribution from "the plasma-blackout examples" to `verification/qtt_ramc_stagline`. Separately, document in the verification README how the +/-0.70-decade band was derived — the word "earned" is doing load-bearing work with no stated derivation, and a back-fitted tolerance would be a category-C defect.

**Adversarial check.** The doc-tier gap is real: README.md:218-219 reads 'The plasma-blackout examples validate an uncalibrated finite-rate ionization network against RAM-C II flight data' with no accuracy tier, while verification/README.md:45-49 explicitly scopes the same gate as 'flight-data, order-of-magnitude ... not a per-point accuracy claim' and the table row at :41 records +0.48 dec inside an earned ±0.70-dec band. However the auditor's supporting sub-claim is REFUTED: the measurement does NOT live only in verification/qtt_ramc_stagline. The corridor example self-verifies against the RAM-C anchor in its own right — corridor/README.md:20 'The run self-verifies against the RAM-C II flight anchor and exits nonzero on any regression' and :65 'the evolved peak electron density lands at 3.4e19 per cubic meter against [the RAM-C station]'. The README's attribution to 'the plasma-blackout examples' is therefore accurate. Also, an order-of-magnitude comparison inside a pre-declared earned band is a validation, just at a coarse tier; the README understates the tier rather than contradicting the result.

> Evidence re-read: deep_causality_cfd/README.md:218-219; verification/README.md:41 (table row, values match verbatim) and :45-52 (scope labels); examples/avionics_examples/cfd/plasma_blackout/corridor/README.md:10,20,65,129,182

---

### 15.5 [MINOR] README presents the shock-fitted inflow strip as answering the rank caveat that the verification README calls a named open remainder

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/README.md:50`
- **Auditor confidence:** confirmed

**Claim.** The crate README states the shock-fitted inflow strip 'answers' the rank caveat without scoping the claim: it holds for the 2-D compressible carrier when a DescentSchedule is attached (a 2-column Dirichlet inlet), and does not bound the marched chi for the blunt-body or 3-D cases the studies measured — which verification/README.md:50-52 records as the named open remainder (design D9).

**Code evidence.**

```
README.md:49-52:
  "The rank studies in `studies/` measured the decisive caveat (the rank driver is coordinate
   alignment, not sharpness), and the compressible carrier answers it with a shock-fitted inflow
   strip: the exact Rankine-Hugoniot state is the boundary of the marched layer, so the shock is
   never captured at all."

verification/README.md:50-52 (same subject, opposite framing):
  "The **dynamic marched**
   rank growth (flux-through-front) and the **wake** are *reported, never asserted* — bounding the marched χ
   needs re-pinning + an exact-RH interface (design D9), the named open remainder."

studies/README.md:42 (qtt_repin_marcher finding):
  "So the Stage-4 lever is re-pin **and** treat the front as an exact Rankine–Hugoniot interface,
   smooth each side, rather than marching fluxes across it."  (stated as the required lever, not a built one)

What the carrier actually does — src/types/flow/compressible_march_run.rs:209-223:
  /// Enforce the shock-fitted inflow strip (Dirichlet over the first `strip_cols` columns).
  fn enforce_inflow(&self, state: &EulerStateTt2d<R>) -> Result<EulerStateTt2d<R>, PhysicsError> {
      let (Some(inflow), Some(schedule)) = (self.inflow, self.schedule.as_ref()) else {
          return Ok(state.clone());          // <-- no-op without a DescentSchedule
      };
      ...
      for i in 0..schedule.strip_cols.min(1usize << self.lx) {   // strip_cols defaults to 2

Cargo default: src/types/flow_config/compressible_march_config.rs:109  strip_cols: 2
```

**Reference form.** verification/README.md:50-52, the crate's own statement of what remains open: "bounding the marched χ needs re-pinning + an exact-RH interface (design D9), the named open remainder."

**Impact.** An engineer scoping whether this crate can carry a shock-dominated case reads the crate README's unconditional "the shock is never captured at all" and concludes the rank problem is solved. It is solved only for the 2-D carrier when a DescentSchedule is attached, where the RH state is imposed as a Dirichlet inlet BC on 2 columns so the shock sits outside the marched domain by construction. That is a real and defensible design, but it is not the in-domain tracked-interface mechanism the studies identified as necessary, and it does not bound the marched chi for the blunt-body or 3-D cases the studies actually measured.

**Recommended fix.** Qualify README.md:49-52 to match the implementation and the verification README: "...the compressible carrier answers it for the descent-schedule case with a shock-fitted inflow strip: the exact Rankine-Hugoniot state is imposed as a Dirichlet inlet over the first `strip_cols` columns, placing the shock at the boundary of the marched layer rather than inside it. Bounding the marched bond dimension in the general (in-domain front) case needs re-pinning plus a tracked exact-RH interface (design D9) and remains open — see verification/README.md." Also note that `enforce_inflow` is a no-op without a `DescentSchedule`.

**Adversarial check.** Both quotes are verbatim and the code reads exactly as cited. enforce_inflow at compressible_march_run.rs:209-223 is a Dirichlet overwrite of the first `strip_cols` columns and returns `Ok(state.clone())` (a no-op) when either `self.inflow` or `self.schedule` is None; strip_cols defaults to 2 at compressible_march_config.rs:109. So the README's 'the shock is never captured at all' is literally true only for the 2-D carrier with a DescentSchedule attached. The verification README:50-52 does name the exact-RH interface (design D9) as 'the named open remainder' and marks the dynamic marched rank 'reported, never asserted', and studies/README.md:42 states the required lever as re-pin AND exact-RH interface. The tension in epistemic status is genuine. But this is a scope/framing omission, not a false statement: the sentence describes a real, implemented mechanism correctly, and the README does not claim it bounds the blunt-body or 3-D marched chi — it only fails to say that it does not.

> Evidence re-read: deep_causality_cfd/README.md:46-52; src/types/flow/compressible_march_run.rs:209-223 (read in full, including the early-return guard); src/types/flow_config/compressible_march_config.rs:48,109,122; verification/README.md:50-52; studies/README.md:42

---

### 15.6 [MAJOR] Five of thirteen verification programs have no per-example section and no reference citation, contrary to the README's own promise

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/verification/README.md:16`
- **Auditor confidence:** confirmed

**Claim.** verification/README.md states twice that every example has a per-example section carrying what it checks, how it fails, and its reference papers. Only 8 of the 13 have one. The 5 without are precisely the compressible/plasma programs, and none of their references — RAM-C II, Park, Sod — appear in the README's References list.

**Code evidence.**

```
verification/README.md:16-17 (the promise):
  "What each one checks, and how it fails, is in the per-example sections below."
verification/README.md:54 (repeated):
  "Reference papers per example are in the sections below and the [References](#references)."

Actual section headers (grep -n "^## \`" verification/README.md) — 8 of 13:
  61:  ## `mms_taylor_green_verification`
  81:  ## `dec_graded_mms_verification`
  99:  ## `dec_taylor_green_re1600_verification`
  120: ## `dec_lid_cavity_re1000_verification`
  139: ## `dec_cylinder_wake_verification`
  159: ## `dec_cylinder_verification`
  183: ## `qtt_taylor_green_verification`
  208: ## `qtt_cylinder_verification`

Missing sections: qtt_park2t_blackout, qtt_sod, qtt_ramc_stagline, qtt_blunt_body_2d, qtt_reentry_3d.
(All five are declared in Cargo.toml:172-190 and all five directories exist with their own README.md.)

The References list, verification/README.md:231-264, holds 15 entries. It contains no RAM-C II
reference, no Park two-temperature reference, and no Sod (1978) reference — i.e. no citation for any
of the five missing examples. The RAM-C citation exists only in the buried per-directory file:
  verification/qtt_ramc_stagline/README.md:75
  "- RAM-C II flight experiment, NASA Langley (1970) — the canonical ionized-reentry electron-density dataset."
```

**Reference form.** verification/README.md:16-17 and :54, the document's own stated structure. For a pre-certification package, every quantitative claim in the summary table needs a traceable citation in the same document.

**Impact.** The five undocumented examples carry the crate's strongest and most contested claims: the RAM-C II flight-data anchor, the Sod exact-Riemann gate that the README itself calls "the only quantitative-accuracy gate", and the two rank-lever structural gates. A reviewer working from verification/README.md gets summary-table numbers for these five with no statement of what the gate is, how it fails, or what published source the reference value comes from. The RAM-C II citation in particular is the single most load-bearing reference in the crate and it is absent from the References section.

**Recommended fix.** Add per-example sections for qtt_park2t_blackout, qtt_sod, qtt_ramc_stagline, qtt_blunt_body_2d and qtt_reentry_3d following the existing template (Verifies / Self-check / Measured / Reference), and add their sources to the References list: RAM-C II (NASA Langley, 1970 — with the specific TN/TM number), Park, C. (1989/1990) two-temperature nonequilibrium model, Sod, G. A. (1978) J. Comput. Phys. 27, 1-31, and the NASA RP-1232 rate table the corridor cites. Alternatively soften lines 16-17 and 54 to name which examples are covered — but for a pre-cert package, adding the sections is the right fix.

**Adversarial check.** `grep -n "^## " verification/README.md` returns exactly 8 per-example sections (61, 81, 99, 120, 139, 159, 183, 208) plus Convention/Summary/References. `ls verification/` returns 13 example directories. The 5 with no section are exactly qtt_park2t_blackout, qtt_sod, qtt_ramc_stagline, qtt_blunt_body_2d, qtt_reentry_3d — all present in the summary table with quantitative values. The promises at :16-17 and :54 are verbatim. I read the entire References block (verification/README.md:231-264): it contains no RAM-C II, no Park, and no Sod entry, so none of the five has a citation in the document that makes the claims. The RAM-C citation exists only in the per-directory file. One immaterial correction: the reference list holds 16 entries, not 15 (the auditor omitted the 2012 HiOCFD Workshop C3.5 entry) — this does not affect the finding.

> Evidence re-read: verification/README.md section-header grep (8 hits); `ls verification/` (13 dirs); verification/README.md:16-17, :54, :231-264 read in full; verification/qtt_ramc_stagline/README.md:75

---

### 15.7 [MAJOR] README claim that the examples are "built entirely from this crate's public API" is false; several types in public signatures are not re-exported

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/README.md:264`
- **Auditor confidence:** confirmed

**Claim.** The README states the end-to-end examples are built entirely from this crate's public API. They import from at least eight sibling crates, and the gap is structural: `Truncation`, `CausalTensor` and `CausalTensorTrain` appear in `deep_causality_cfd`'s own public method signatures but are absent from its `pub use` surface, so a consumer cannot call the config builders without adding an undeclared direct dependency.

**Code evidence.**

```
README.md:262-264:
  "The end-to-end examples, the plasma-blackout corridor and its weather-dispersion
   table, live in [`examples/avionics_examples/cfd/`](../examples/avionics_examples/cfd/). They
   are built entirely from this crate's public API."

Actual imports in examples/avionics_examples (cfd/ and src/), non-cfd deep_causality crates:
  use deep_causality_core::AlternatableContext;        <- corridor/main.rs:40, required to compile the README's own snippet
  use deep_causality_core::{CausalFlow, CausalityError, CausalityErrorEnum, PropagatingEffect};
  use deep_causality_core::EffectLog;
  use deep_causality_tensor::Truncation;
  use deep_causality_tensor::CausalTensor;
  use deep_causality_topology::HodgeDecomposeOptions;
  use deep_causality_calculus::{EndoArrow, Rk4, Scalar};
  use deep_causality_algebra::Real;
  use deep_causality_num::{Float106, FromPrimitive};
  use deep_causality_haft::LogSize;

src/lib.rs re-exports only PhysicsError + physics quantities, deep_causality_file items, and
deep_causality_haft::IoAction. `grep -c "Truncation\|CausalTensor\|EffectLog\|AlternatableContext" src/lib.rs` == 0.

Yet these types are unavoidable in the crate's own public API:
  src/types/flow_config/qtt_march_config.rs:211        pub fn solver(mut self, dt: R, nu: R, trunc: Truncation<R>) -> Self
  src/types/flow_config/compressible_march_config.rs:382 pub fn solver(mut self, dt_solver: R, s_ref: R, gamma: R, trunc: Truncation<R>) -> Self
  src/types/flow_config/qtt_march_config.rs:292        pub fn seed_fields(mut self, u0: CausalTensor<R>, v0: CausalTensor<R>) -> Self
  src/types/flow/qtt_march_run.rs:629                  pub fn u(&self) -> &CausalTensorTrain<R>
  src/types/flow/march_run.rs:327                      pub fn one_form(&self) -> &CausalTensor<R>

The README's own second code snippet (README.md:137) uses `.alternate_context(&committed)`, which
resolves through `deep_causality_core::AlternatableContext` — a trait the reader must import from a
crate the README never mentions.
```

**Reference form.** Rust API guidelines C-REEXPORT: "the crate makes all types that appear in its public API available at the crate root", and the README's own sentence at line 264.

**Impact.** Two concrete consequences. First, a consumer who adds only `deep_causality_cfd` as a dependency, per the README's Usage section at lines 26-28, cannot construct a `QttMarchConfig` or `CompressibleMarchConfig` at all — `Truncation` is unnameable. They must discover and add `deep_causality_tensor` at a version they have to guess, since the cfd Cargo.toml pins it at path+0.5. Second, the README's own counterfactual snippet does not compile as printed without `use deep_causality_core::AlternatableContext;`, which appears nowhere in the README.

**Recommended fix.** Re-export the types that leak into the public API from src/lib.rs: `pub use deep_causality_tensor::{CausalTensor, CausalTensorTrain, Truncation};` and `pub use deep_causality_core::{AlternatableContext, AlternatableState, AlternatableValue, EffectLog};`. Then either the README claim becomes true, or restate line 264 as "They are built from this crate's public API plus the core causal-monad vocabulary (`deep_causality_core::AlternatableContext`)." Add the missing `use` line to the README snippet at line 131-141 so it compiles as printed.

**Adversarial check.** README.md:262-264 carries the sentence verbatim. I dumped every `use deep_causality_*` line across examples/avionics_examples/cfd and src: they import from deep_causality_core (AlternatableContext, EffectLog, CausalFlow, PropagatingEffect), deep_causality_tensor (Truncation, CausalTensor), deep_causality_topology, deep_causality_calculus, deep_causality_algebra, deep_causality_num, deep_causality_haft, deep_causality_physics — eight sibling crates. The structural half checks out too: `grep -c 'Truncation|CausalTensor|EffectLog|AlternatableContext' src/lib.rs` == 0, while all four cited signatures exist as quoted — qtt_march_config.rs:211 and compressible_march_config.rs:382 both take `trunc: Truncation<R>`, qtt_march_config.rs:292 takes `CausalTensor<R>`, qtt_march_run.rs:629 returns `&CausalTensorTrain<R>`. A consumer adding only deep_causality_cfd genuinely cannot name Truncation and so cannot build either march config.

> Evidence re-read: deep_causality_cfd/README.md:262-264; src/lib.rs:32-125 read in full (re-export surface); src/types/flow_config/qtt_march_config.rs:211,292; src/types/flow_config/compressible_march_config.rs:382; src/types/flow/qtt_march_run.rs:629; full `use deep_causality_*` inventory over examples/avionics_examples

---

### 15.8 [MINOR] Traceability artifacts are largely absent: papers/ holds 4 PDFs against ~20 cited works, and Cargo.toml excludes them plus all tests from the package

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/README.md:260`
- **Auditor confidence:** confirmed

**Claim.** papers/ holds 4 PDFs against 16 cited works. Two (kirkpatrick2003, Droege2005) ARE traceably the sources behind a shipped closure and a reference value, per docstring citations at src/solvers/dec/surface_force.rs:72-75,166 and verification/dec_cylinder_verification/main.rs:35; two (mittal2005, mohamed2016) have no citation in cfd src/ or verification/. The uncovered sources include Park, RAM-C II, and Millikan-White. Cargo.toml:18-29 excludes papers/* and tests/**/* against README.md:213's 'ships its evidence'.

**Code evidence.**

```
README.md:260:
  | `papers/` | The cited source PDFs behind constants and closures |

`ls papers/` — the complete contents:
  Droege2005.pdf
  kirkpatrick2003.pdf
  mittal2005.pdf
  mohamed2016.pdf

verification/README.md:231-264 cites 15 works. Of those, only Dröge & Verstappen (2005) is present.
Absent: Taylor & Green (1937), Ghia/Ghia/Shin (1982), Williamson (1996), Peddinti et al. (2024),
Gourianov et al. (2022), Angot/Bruneau/Fabrie (1999), Brachet et al. (1983), van Rees et al. (2011),
Hirani (2003), Desbrun et al. (2005), Regge (1961), Roache (2002), Salari & Knupp (2000),
Lehmkuhl et al. (2013).

Also absent are the sources behind the actual physics constants the README names:
  - Park two-temperature closure (README.md:54-55, and PARK_NO_IONIZATION_* constants imported at
    src/solvers/qtt/compressible/fitting.rs:21-22)
  - RAM-C II flight data (README.md:218-219)
  - Millikan-White relaxation (README.md:163)
  - NASA RP-1232 rate table (cited in the corridor's Limitations section)

Cargo.toml:18-29:
  exclude = [
      ... "tests/**/*", "papers/*",
  ]
against README.md:213 "The crate ships its evidence."
Note the Couette/Poiseuille validations the README cites at line 42 live in tests/, which is excluded.
```

**Reference form.** The user's own stated convention for this repository (cite papers in kernel docstrings with the PDF in the papers/ folder), and README.md:260's description of what papers/ contains.

**Impact.** For a pre-certification audit, papers/ is the crate's declared traceability mechanism, and it covers roughly one in five cited works and none of the constants an auditor would most want to trace — Park's ionization rates, the Millikan-White fit, and the RAM-C II dataset. The Cargo exclude compounds it: if the crate is ever published (currently `publish = false` at Cargo.toml:32, pending this audit), a downstream consumer receives neither the papers nor the tests that hold the Couette and Poiseuille validations, while the README tells them the crate ships its evidence.

**Recommended fix.** Either add the missing PDFs (at minimum Park, RAM-C II, Millikan-White, RP-1232, Ghia 1982, Taylor-Green 1937, Sod 1978) or restate README.md:260 to describe what is actually there, e.g. "| `papers/` | Source PDFs for the immersed-boundary and DEC-operator closures (Droege 2005, Kirkpatrick 2003, Mittal 2005, Mohamed 2016). Other cited works are listed in verification/README.md. |". Before flipping `publish = true`, remove `papers/*` from the exclude list, or amend README.md:213 to say the evidence lives in the git repository rather than the published package.

**Adversarial check.** The counts are right: `ls papers/` returns exactly Droege2005, kirkpatrick2003, mittal2005, mohamed2016 against 16 works in verification/README.md:231-264, and Cargo.toml:18-29 does exclude both `tests/**/*` and `papers/*` while README.md:213 says 'The crate ships its evidence'. The Couette/Poiseuille validations the README cites at line 42 do live under the excluded tests/ (tests/solvers/dec/poiseuille_tests.rs, tests/theories/incompressible_ns_verification_tests.rs). But the auditor's claim that 'none of [the four PDFs] are the sources behind the crate's headline constants or closures' is REFUTED: kirkpatrick2003 is precisely the source for the wall-traction closure, cited in the kernel docstring at src/solvers/dec/surface_force.rs:72-75 ('the Kirkpatrick wall traction t = mu S.n') and again at the implementation line 166 — this is the repo's stated cite-in-docstring + PDF-in-papers convention working as designed. Droege2005 likewise backs the cylinder drag reference (verification/dec_cylinder_verification/main.rs:35). mittal2005 and mohamed2016 have no matching citation in cfd src/ or verification/, so 2 of 4 PDFs are orphaned.

> Evidence re-read: `ls deep_causality_cfd/papers/` (4 files); deep_causality_cfd/Cargo.toml:18-29 and :32; README.md:213, :260; verification/README.md:231-264 (16 entries); src/solvers/dec/surface_force.rs:72,75,166; verification/dec_cylinder_verification/main.rs:35; tests/solvers/dec/poiseuille_tests.rs

---

### 15.9 [MINOR] The entire powered-descent/retropulsion, duct, snapshot, and operator-study public surface is undocumented outside its own source files

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/README.md:246`
- **Auditor confidence:** confirmed

**Claim.** The core claim (undocumented public surface: retropulsion/duct/snapshot/operator-study/KeyedTable/AcousticCoreInverse) is confirmed, as is the two-vs-seven example gap. The 'three regime axes' sub-claim is REFUTED: README.md:107-122's three axes are flow / dynamics / link — the dynamics axis is the navigation engine's Encke-Cowell integrator switch (src/navigation), which is not a component of RegimeClass::key() at all. key()'s 5-tuple is a different taxonomy (model, gnss_denied, mach_regime, thrust_state, touchdown); the two lists are not comparable and the README is not wrong here.

**Code evidence.**

```
Checked each name against README.md and the crate rustdoc block src/lib.rs:6-24. All scored 0/0:
  DuctMarchRun, DuctConfig, DuctInlet, DuctAreaProfile, DuctStop,
  IgnitionCorridor, ThrottleGuidance, RetroThrust, PlumeNozzle, PlumeObstruction, BurnEnvelope,
  RecoveryTemperatureStage, ViscosityArrhenius, EosStage, ThrustState, MachRegime,
  save_resume_state, load_resume_state, pack_resume, unpack_resume, NamedTtFields,
  Operator, OperatorStudyBuilder, KeyedTable, KeyedInterpolation,
  AcousticCoreInverse (+2d/3d), ForkEconomics, FlightSensors, DescentSchedule, AtmosphereRow,
  ImuModel, MmsBuilder, Gates, ThermalRelax

All are exported: src/lib.rs:49, 57, 66, 74-77, 90-112, 114-120.

The README's "Where Things Live" entry for src/types/flow/ (README.md:252) enumerates
  "the trajectory march (runs, pauses, forks, the named-stage builder) and the campaign study grammar
   (phase family, `GateSeq`/`Verdict`, the `StudyEffect` carrier, the `save_log` audit sink), plus the
   coupling stack, physics stages, blackout stages, and reports"
but `ls src/types/flow/` also contains retropulsion.rs, throttle_guidance.rs, duct_march_run.rs,
state_snapshot.rs, operator_study.rs, flight_sensors.rs, frequency.rs — none described.

Relatedly, README.md:262-263 names only two end-to-end examples ("the plasma-blackout corridor and
its weather-dispersion table"), but examples/avionics_examples/Cargo.toml declares seven CFD binaries:
  turbulence_flow, plasma_blackout_corridor, plasma_blackout_weather, plasma_blackout_retropulsion,
  flight_envelope_placard, nozzle_operating_map, viv_resonance_margin.

And README.md:107-122 "Native Multi Regime" says "this crate switches three regime axes independently",
but RegimeClass::key() (src/types/flow/corridor/regime.rs:127-135) switches on five discrete axes:
  (self.model, self.gnss_denied, self.mach_regime, self.thrust_state, self.touchdown)
```

**Reference form.** Rust API guidelines C-CRATE-DOC: the crate root documentation should orient a reader to the crate's major capabilities. The README's own "Where Things Live" table (README.md:246-260) is the crate's stated discovery mechanism.

**Impact.** The retropulsion subsystem is a substantial capability with a dedicated end-to-end example (plasma_blackout_retropulsion) and a dedicated study (srp_momentum_jet), yet a reader of the README would not know powered descent is supported at all. Likewise, `save_resume_state`/`load_resume_state` is a cross-workflow checkpoint/resume capability — precisely the kind of thing a lab evaluating the crate for long-running analyses needs to find — and it is invisible above the source level. For a pre-cert review, undocumented capability is as much a finding as overclaimed capability: it means the documented scope does not match the certified artifact.

**Recommended fix.** Add these lines to the README (and mirror the first three into the src/lib.rs crate rustdoc):
• Powered descent and retropulsion: `RetroThrust` writes commanded thrust into the force channel while `PlumeObstruction` applies the plume drag decrement and optionally publishes the analytic Cordell-Braun plume geometry from a `PlumeNozzle`; `ThrottleGuidance` commits through a four-condition `IgnitionCorridor`, and `BurnEnvelope` extends `SafetyEnvelope` with the throttle, thrust-coefficient, propellant, and descent-rate limits the cybernetic gate enforces.
• Quasi-one-dimensional duct marching: `DuctConfig` (geometry, inlet stagnation state, back pressure, resolution, stop condition) is run by `CfdFlow::march` into a `DuctMarchRun`, for nozzle and inlet operating maps.
• Checkpoint and resume: `save_resume_state` / `load_resume_state` pack a running coupled state (including tensor-train fields via `pack_tt_fields`) to a versioned snapshot file, so a long march can be stopped and continued in a different program.
• Auxiliary closures and studies: `RecoveryTemperatureStage`, `ViscosityArrhenius`, and `EosStage` are between-step closures; `CfdFlow::operator_study` sweeps a DEC `Operator` over a resolution ladder and reports the observed convergence order; `KeyedTable`/`KeyedInterpolation` provide key-bracketed linear interpolation with end clamping for atmosphere and property tables; `AcousticCoreInverse{,2d,3d}` is the closed-form ADI-split inverse of the constant-coefficient acoustic core used to precondition the implicit step.
Also: update README.md:262-263 to name all seven CFD examples, and correct README.md:110-111 from "three regime axes" to five (flow model, comms denial, Mach band, thrust state, touchdown) plus the navigation integrator regime.

**Adversarial check.** Case-insensitive counts against README.md: Duct 0, Retro 0, Throttle 0, snapshot 0, resume 0, KeyedTable 0, Acoustic 0, Ignition 0, retropulsion 0. All the named items are genuinely public — DuctMarchRun/BurnEnvelope/IgnitionCorridor/RetroThrust/PlumeNozzle/PlumeObstruction/ThrottleGuidance/RecoveryTemperatureStage/EosStage/ThrustState/MachRegime/Operator/OperatorStudyBuilder/Gates/MmsBuilder/FlightSensors/ForkEconomics/ThermalRelax/ViscosityArrhenius at src/lib.rs:104-120, save_resume_state/load_resume_state/pack_resume/unpack_resume/NamedTtFields at :56-59, AcousticCoreInverse(+2d/3d) at :39, KeyedTable/KeyedInterpolation at :49, AtmosphereRow/DescentSchedule at :113. The README's src/types/flow/ row (:252) enumerates the flow layer without naming any of them, and README.md:262-263 names two end-to-end examples while examples/avionics_examples/Cargo.toml declares seven CFD binaries. The discovery-layer gap is exactly as described. One sub-claim corrected below.

> Evidence re-read: deep_causality_cfd/README.md keyword counts (all 0) and :252, :262-263; src/lib.rs:39,49,56-59,104-120 (export list read in full); examples/avionics_examples/Cargo.toml:13-57 (7 CFD bins); src/types/flow/corridor/regime.rs:127-135

---

### 15.10 [MINOR] README documents field.regime() with the wrong return type and only 4 of its 7 fields

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/README.md:128`
- **Auditor confidence:** confirmed

**Claim.** The README's inline signature comment states `field.regime()` returns `Option<&RegimeClass>` with four fields. It returns an owned `Option<RegimeClass<R>>`, the type is generic over the scalar, and it carries seven public fields.

**Code evidence.**

```
README.md:128:
  // `field.regime()` -> Option<&RegimeClass> { model, knudsen, plasma_frequency, gnss_denied }.

Actual signature, src/types/flow/coupling.rs:274-277:
  /// The last governing-model regime the classifier selected, if a classifier stage has run.
  pub fn regime(&self) -> Option<RegimeClass<R>> {
      self.regime
  }

Actual fields, src/types/flow/corridor/regime.rs:103-118:
  pub struct RegimeClass<R: CfdScalar> {
      pub model: GoverningModel,
      pub knudsen: R,
      pub plasma_frequency: R,
      pub gnss_denied: bool,
      pub mach_regime: MachRegime,        // undocumented in README
      pub thrust_state: ThrustState,      // undocumented in README
      pub touchdown: bool,                // undocumented in README
  }
```

**Reference form.** The definition itself: `pub fn regime(&self) -> Option<RegimeClass<R>>` at coupling.rs:275, returning the `Copy` struct by value, and the 7-field struct at regime.rs:103-118.

**Impact.** The borrow-vs-owned difference is small in practice because `RegimeClass` is `Copy`, but the three omitted fields are the powered-descent axes and they are the discoverable entry point to the whole retropulsion subsystem. A reader who writes a `.until(...)` predicate against the README's four-field view will not know that `r.thrust_state`, `r.mach_regime` and `r.touchdown` are available as stop conditions.

**Recommended fix.** Change README.md:128 to: `// field.regime() -> Option<RegimeClass<R>> { model, knudsen, plasma_frequency, gnss_denied, mach_regime, thrust_state, touchdown }.` and add one sentence noting that the last three are the powered-descent axes, neutral unless a world publishes the corresponding flight scalars and `RegimeClassify::with_flight_axes` is attached.

**Adversarial check.** README.md:128 reads verbatim `// \`field.regime()\` -> Option<&RegimeClass> { model, knudsen, plasma_frequency, gnss_denied }.` The definition at src/types/flow/coupling.rs:274-276 is `pub fn regime(&self) -> Option<RegimeClass<R>> { self.regime }` — owned, not a reference, and generic over R. The struct at regime.rs:103-118 has exactly seven pub fields; the three the README omits (mach_regime, thrust_state, touchdown) are each individually documented in the struct. Nothing elsewhere in the README corrects either point. Severity 'minor' is right — RegimeClass is Copy (derive at regime.rs:102), so the borrow-vs-owned slip is harmless, and the README's own adjacent code snippets use `.map(|r| r.gnss_denied)` which compiles either way.

> Evidence re-read: deep_causality_cfd/README.md:128 and the two .until() snippets at :133-142; src/types/flow/coupling.rs:274-277; src/types/flow/corridor/regime.rs:102-118

---

### 15.11 [MINOR] RegimeClassify's own rustdoc understates its change-detection key by three axes

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/src/types/flow/corridor/regime.rs:141`
- **Auditor confidence:** confirmed

**Claim.** The `RegimeClassify` struct rustdoc says it logs a provenance entry "whenever the regime (governing model or comms-denial) changes". The change-detection key it actually compares includes three further axes, so the stage logs transitions the documentation does not mention.

**Code evidence.**

```
src/types/flow/corridor/regime.rs:138-142 (the struct rustdoc):
  /// The governing-model selector (\[2\]/\[3\]). Reads the peak mean free path from a `"mean_free_path"`
  /// field and forms `Kn = λ / L` ... then records the [`RegimeClass`] on the field — logging a
  /// provenance entry whenever the regime (governing model or comms-denial) changes.

src/types/flow/corridor/regime.rs:127-135 (the key actually compared):
  fn key(&self) -> (GoverningModel, bool, MachRegime, ThrustState, bool) {
      (
          self.model,
          self.gnss_denied,
          self.mach_regime,
          self.thrust_state,
          self.touchdown,
      )
  }

src/types/flow/corridor/regime.rs:315:
  let changed = field.regime().map(|prev| prev.key()) != Some(class.key());
```

**Reference form.** The `key()` method at regime.rs:127-135, which is the sole determinant of whether a log entry is emitted.

**Impact.** Bounded, because the doc comment on `key()` itself (regime.rs:121-126) is accurate and explains that the three flight axes are constant for a world publishing no flight scalars, so the corridor's logged transitions are unchanged. But a reader of the struct doc who attaches flight axes will see log entries the struct doc says cannot occur, and any consumer counting or parsing regime log lines will under-predict the entry count.

**Recommended fix.** Amend regime.rs:141-142 to: "logging a provenance entry whenever the regime changes, where a regime is the tuple (governing model, comms-denial, Mach band, thrust state, touchdown) — the last three neutral unless `with_flight_axes` is attached. The transition count is also published typed as `REGIME_TRANSITIONS_FIELD`."

**Adversarial check.** The struct rustdoc at regime.rs:138-142 reads verbatim 'logging a provenance entry whenever the regime (governing model or comms-denial) changes'. The sole gate on emission is regime.rs:315 `let changed = field.regime().map(|prev| prev.key()) != Some(class.key());` and key() at :127-135 returns the 5-tuple (model, gnss_denied, mach_regime, thrust_state, touchdown). So a mach-band, thrust-state, or touchdown transition alone does emit an entry the struct doc says cannot occur. I checked for compensation and found partial mitigation the auditor already credits: key()'s own doc comment at :121-126 is accurate and states the three flight axes are constant for a world publishing no flight scalars. The log line at :338-353 even appends a `, {mach} / {thrust}[, touchdown]` phase suffix when those axes are live — further evidence the struct doc, not the code, is stale. Severity minor is correct.

> Evidence re-read: src/types/flow/corridor/regime.rs:121-126 (key doc), :127-135 (key body), :138-142 (struct rustdoc), :308-320 (change gate), :338-353 (format + phase suffix)

---

### 15.12 [MINOR] Public Gates::finish() prints to stdout, sitting beside the README's blanket "the DSL never exits or prints"

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/src/types/flow/gates.rs:43`
- **Auditor confidence:** confirmed

**Claim.** The README states "the DSL never exits or prints". That is true of the study grammar's `GateSeq`/`verdict()` path, but the crate also exports a similarly-named public type `Gates` whose terminal method writes five distinct lines to stdout. The README does not disambiguate the two.

**Code evidence.**

```
README.md:104: "the DSL never exits or prints (`verdict()` returns data)"

src/types/flow/gates.rs:42-64 — the only println! in the entire crate:
  pub fn finish(self) -> bool {
      println!("--- {} ---", self.title);
      let mut all = true;
      for (label, pass, detail) in &self.entries {
          println!("  [{}] {label}: {detail}", if *pass { "PASS" } else { "FAIL" });
          all &= *pass;
      }
      if self.entries.is_empty() {
          println!("=== {}: no gates registered. ===", self.title);
      } else if all {
          println!("=== All gates passed: {}. ===", self.title);
      } else {
          println!("=== Gate REGRESSION in {}: see the FAIL lines above. ===", self.title);
      }
      all
  }

`Gates` is exported at src/lib.rs:95, one line away from `GateSeq` at src/lib.rs:108.
The crate has no `process::exit` anywhere, so the "never exits" half of the claim holds unconditionally.
```

**Reference form.** README.md:104's own statement, read against the crate's full public surface.

**Impact.** Low but real, and it is a naming hazard rather than a behavioral bug. `Gates` and `GateSeq` are adjacent in the export list and only one of them honors the documented no-print contract. A consumer embedding this crate in a service — where stray stdout is unwelcome — has no README-level signal that one of the two gate types writes to it.

**Recommended fix.** Qualify README.md:104 to "the study grammar never exits or prints (`verdict()` returns data); the separate `Gates` helper is the opt-in `[PASS]`/`[FAIL]` printer the self-verifying programs use". The gates.rs module header at lines 6-12 already documents the printing behavior accurately, so only the README needs the disambiguation.

**Adversarial check.** README.md:104 reads verbatim 'the DSL never exits or prints (`verdict()` returns data)'. gates.rs:43-64 contains five println! calls exactly as quoted, and `grep -rn 'println!' src/ | wc -l` returns 5 — all inside this one function, so the auditor's 'the only println! in the entire crate' is precise, not an approximation. `Gates` is exported at src/lib.rs alongside GateSeq in the same `pub use crate::types::flow::{...}` block, so the naming adjacency hazard is real. The 'never exits' half holds: no process::exit anywhere. Severity minor is right — finish() is documented at gates.rs:39-41 as printing, so the item-level doc is honest; only the README's blanket sentence overreaches.

> Evidence re-read: deep_causality_cfd/README.md:104; src/types/flow/gates.rs:32-64 (full finish() body plus its own rustdoc); println! count across src/ = 5; src/lib.rs export block containing both Gates and GateSeq

---

### 15.13 [MINOR] SolenoidalField's rustdoc claim "there is no path that re-wraps a modified tensor" is false — two public methods write arbitrary coefficients

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_physics/src/quantities/fluid_dynamics/solenoidal_field.rs:20`
- **Auditor confidence:** confirmed

**Claim.** The type's module rustdoc asserts that projection is the only way into the type and that no path re-wraps a modified tensor. Two `pub` methods — both labelled "Crate-internal" in their own doc comments despite being fully public — consume a SolenoidalField and return one with caller-chosen coefficients written into it. The type is re-exported through deep_causality_cfd, so this is reachable from CFD consumer code.

**Code evidence.**

```
solenoidal_field.rs:18-22 (the claim):
  //! "You cannot time-step an unprojected field" is thereby a compile-time
  //! fact: the type has no other constructor, and **no arithmetic** ... Read access is provided by
  //! [`SolenoidalField::as_one_form`]; there is no path that re-wraps a
  //! modified tensor.

solenoidal_field.rs:219-232 — a public path that writes arbitrary values at arbitrary indices:
  /// Crate-internal wall-bounded path: set the prescribed tangential wall
  /// values (the moving-wall lift — edge index → edge integral).
  pub fn with_lift(self, lift: &[(usize, R)]) -> Self {
      if lift.is_empty() { return self; }
      let mut data = self.field.into_vec();
      for &(e, value) in lift {
          data[e] = value;            // <- caller-chosen value at caller-chosen index
      }
      let len = data.len();
      Self { field: CausalTensor::new(data, alloc::vec![len]).expect(...) }
  }

solenoidal_field.rs:198-211 — `pub fn constrain_edges(self, edges: &[usize]) -> Self` similarly zeroes arbitrary indices.

Both doc comments say "Crate-internal" but neither is `pub(crate)`.

Reachability from CFD: deep_causality_physics/src/quantities/mod.rs:57 `pub use ...::SolenoidalField;`
and deep_causality_cfd/src/lib.rs:42 `pub use deep_causality_physics::quantities::*;`
```

**Reference form.** The module's own rustdoc at solenoidal_field.rs:18-22, and the type-state pattern it invokes: a type-state is sound only when every public path preserves the invariant.

**Impact.** The crate README's claim 1 (README.md:41-42, "the SolenoidalField type-state rejects time-stepping an unprojected field at compile time") survives on its own terms — `Marcher::State = SolenoidalField` (src/solvers/dec/marcher.rs:17-19) and there is no public raw constructor, so a `VelocityOneForm` genuinely cannot be time-stepped. But the stronger rustdoc claim does not: a consumer can take a validly projected field, call `.with_lift(&[(0, 1e9)])`, and time-step a field that is no longer divergence-free, with no compile error and no runtime check. The methods' own doc comments say "Crate-internal", so the visibility appears unintentional.

**Recommended fix.** Make `constrain_edges` and `with_lift` `pub(crate)` — their doc comments already declare that intent and a workspace-wide build will show whether any out-of-crate caller exists. If they must stay public (the CFD solver lives in a different crate, so `pub(crate)` will not reach), move them behind a `#[doc(hidden)]` or a sealed extension trait, and correct solenoidal_field.rs:18-22 to state the real invariant: construction requires a projection, and the two wall-boundary paths are the sanctioned exceptions whose divergence perturbation the wall-bounded-ns spec bounds.

**Adversarial check.** The module rustdoc at solenoidal_field.rs:18-22 reads verbatim 'Read access is provided by [SolenoidalField::as_one_form]; there is no path that re-wraps a modified tensor.' Both counterexamples are exactly as cited and both are plain `pub`: with_lift at :213-231 does `data[e] = value` at caller-supplied (index, value) pairs then rebuilds `Self { field: CausalTensor::new(...) }`; constrain_edges at :198-211 zeroes caller-supplied indices the same way. Both doc comments open with 'Crate-internal', which is a comment, not a visibility modifier — no pub(crate), no sealed-token argument, no #[doc(hidden)]. I checked the surrounding module for a compensating runtime re-projection and found none: as_one_form at :234-237 even states 'There is no mutable or re-wrapping counterpart by design', which the two methods above contradict. Reachability through deep_causality_cfd is as stated. Severity minor is right because the auditor correctly scopes it: the README's weaker claim-1 (no time-stepping an unprojected field) survives, since Marcher::State = SolenoidalField and no raw constructor exists.

> Evidence re-read: deep_causality_physics/src/quantities/fluid_dynamics/solenoidal_field.rs:10-22, :190-211 (constrain_edges), :213-231 (with_lift), :234-237 (as_one_form doc); deep_causality_cfd/src/lib.rs `pub use deep_causality_physics::quantities::*;`

---

### 15.14 [MINOR] The "chi^2 * L, logarithmic in point count" cost claim omits that the crate's own studies measured chi growing as sqrt(side)

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/README.md:47`
- **Auditor confidence:** confirmed

**Claim.** The README states a 2^L grid costs order chi^2 * L, "logarithmic in point count". That is the correct storage scaling for a fixed bond dimension, but the crate's own rank studies found chi is not fixed for the shock-dominated cases the QTT marchers target — it grows as sqrt(side) in 3-D, making the real cost linear in side length, not logarithmic.

**Code evidence.**

```
README.md:46-49:
  "**Compression-based: the QTT marchers.** The compressible Euler marchers (1-D through 3-D,
   including a body-fitted variant) run on quantized tensor trains, where a `2^L` grid costs order
   `chi^2 * L`: logarithmic in point count, with sharp structure paid for in bond dimension."

studies/README.md:28 (the crate's own 3-D measurement):
  "A realistically-formed 3-D curved shock, via explicit Euler and central differences, has
   **χ ~ √side, unbounded**, running 45 → 135 over 16³ → 128³; flat and body-fitted stay **χ ~ 6,
   constant**. QTT storage still beats dense asymptotically, with a crossover near 64³, but the
   √side **solve** cost is what bites."

studies/README.md:25:
  "A *captured* misaligned shock is net-negative, at χ ≈ 151–394, larger than dense."
```

**Reference form.** Standard MPS/TT storage scaling, O(L · d · chi^2) with d = 2 for a quantized binary grid (Oseledets 2011, "Tensor-Train Decomposition", SIAM J. Sci. Comput. 33(5); and Gourianov et al. 2022, cited at verification/README.md:237-239, which states memory ~ O(D^2 log N)). The formula is right; the claim it supports depends on chi being bounded.

**Impact.** Bounded, because the README does hedge with "sharp structure paid for in bond dimension" in the same sentence, and the studies README states the caveat plainly. But an engineer sizing a 3-D case from the README's headline would compute a logarithmic budget and be off by a factor of sqrt(side) — the studies measured chi running 45 to 135 across 16^3 to 128^3, and note that QTT can be net-negative versus dense for a captured misaligned shock. The README also says "costs" without distinguishing storage from solve cost; the studies explicitly identify solve cost as the binding constraint.

**Recommended fix.** Tighten README.md:46-49 to separate the two: "...where a 2^L grid *stores* order chi^2 * L: logarithmic in point count for a bounded chi. Whether chi stays bounded is the design question the rank studies answered — it does in a shock-aligned or body-fitted coordinate (chi ~ 5-6, flat in resolution), and it does not for a captured misaligned shock (chi ~ sqrt(side), 45 to 135 across 16^3 to 128^3, net-negative versus dense). Solve cost, not storage, is the binding constraint. See studies/README.md."

**Adversarial check.** I re-derived the reference form and the auditor's is correct: TT/MPS storage for a quantized binary grid is O(L·d·chi^2) with d=2, i.e. O(chi^2 · log N) only for bounded chi — so the README's formula at :47 is right and the claim it supports is conditional on chi. studies/README.md:28 measures the opposite condition for the target regime: 'a realistically-formed 3-D curved shock ... has chi ~ sqrt(side), unbounded, running 45 -> 135 over 16^3 -> 128^3', and :25 records a captured misaligned shock at chi 151-394, 'larger than dense'. The studies text also identifies the solve cost, not storage, as binding ('the sqrt(side) solve cost is what bites'), while the README says only 'costs'. Confirmed at the auditor's own severity — the README does hedge in the same sentence with 'sharp structure paid for in bond dimension', which is why this is minor rather than a false claim.

> Evidence re-read: deep_causality_cfd/README.md:46-52; studies/README.md:25 (qtt_rank_study row) and :28 (qtt_rank_3d row), both read in full; verification/README.md:237-239 (Gourianov 2022 entry present as cited)

---

### 15.15 [INFO] README's quoted regime-transition log lines are column-aligned in a way the format string cannot produce

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/README.md:148`
- **Auditor confidence:** confirmed

**Claim.** The README presents four log lines as verbatim output "From an actual corridor run", linking output.txt. One line carries padding whitespace that the format string does not emit, so the block is not a faithful transcript.

**Code evidence.**

```
README.md:146-151, introduced at :143-144 as "Every transition lands in the provenance log. From an actual corridor run ([output.txt](...))":
  regime -> slip (GNSS-available), Kn=0.07829109848665225
  regime -> slip (GNSS-denied),    Kn=0.01705925949914955      <- four spaces after the comma
  regime -> continuum (GNSS-denied), Kn=0.009938308574526865
  regime -> continuum (GNSS-available), Kn=0.00025839060489290773

The format string, src/types/flow/corridor/regime.rs:346-353:
  field.log_mut().add_entry(&format!(
      "regime -> {} ({}), Kn={}{}",
      model.name(), denial, kn, phase,
  ));
— a single space after the comma, unconditionally.

The committed artifact agrees with the code, not the README:
  corridor/output.txt:58   regime -> slip (GNSS-denied), Kn=0.01705925949914955
```

**Reference form.** The format string at regime.rs:347, `"regime -> {} ({}), Kn={}{}"`, and the committed output.txt:56-67.

**Impact.** Cosmetic. The model names, denial strings, and all four Knudsen values match the committed run exactly, so the substance is correct and the log format claim holds. Recording it only because the block is explicitly framed as a verbatim transcript of a linked artifact, and in a pre-certification package a transcript that has been hand-adjusted is worth knowing about even when the adjustment is whitespace.

**Recommended fix.** Copy the four lines verbatim from corridor/output.txt:56-67, dropping the added alignment padding on line 148.

**Adversarial check.** Byte-level inspection of README.md:147-150 shows line 148 as `regime -> slip (GNSS-denied),    Kn=0.01705925949914955` — four spaces after the comma, where lines 147/149/150 have one. The format string at regime.rs:346-353 is `"regime -> {} ({}), Kn={}{}"` with a single unconditional space, and the trailing {} is the phase suffix (empty when no flight scalars are published), not padding. The committed artifact agrees with the code: corridor/output.txt:58 has one space. The block is introduced at README.md:143-144 as 'From an actual corridor run ([output.txt](...))'. Severity info is right: all four model names, both denial strings, and all four Kn values match the artifact digit-for-digit.

> Evidence re-read: deep_causality_cfd/README.md:145-152 inspected with per-line delimiters; src/types/flow/corridor/regime.rs:338-353 (format string plus the phase-suffix branch); examples/avionics_examples/cfd/plasma_blackout/corridor/output.txt:54-70

---

### 15.16 [INFO] studies/README.md points at reverted/srp_drag_decrement under a stale verification/ path

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/studies/README.md:60`
- **Auditor confidence:** confirmed

**Claim.** The studies README refers a reader to `verification/srp_drag_decrement/`. No such directory exists; the superseded harness lives at `reverted/srp_drag_decrement/`.

**Code evidence.**

```
studies/README.md:58-60:
  "`srp_momentum_jet` is
   the imprint-fidelity follow-up (risk 1); it supersedes the reverted
   `verification/srp_drag_decrement/` pinned-envelope harness (see `reverted/README.md`)."

`ls verification/` — 13 directories, none named srp_drag_decrement.
`ls reverted/` — README.md, srp_drag_decrement/

reverted/README.md:16 lists it as `srp_drag_decrement/` (no verification/ prefix), superseded by
`studies/srp_momentum_jet/`.
```

**Reference form.** The actual filesystem layout: reverted/srp_drag_decrement/, per reverted/README.md:16.

**Impact.** Minimal — the same sentence points to reverted/README.md, so a reader recovers immediately. Worth fixing because the reverted/ folder is the repository's provenance mechanism for superseded verdicts, and a broken path into it weakens exactly the audit trail it exists to provide.

**Recommended fix.** Change studies/README.md:60 to `reverted/srp_drag_decrement/`. If the intent was to record where the harness originally lived, say so explicitly: "it supersedes the pinned-envelope harness now parked at `reverted/srp_drag_decrement/` (originally `verification/`)".

**Adversarial check.** studies/README.md:58-60 reads verbatim 'it supersedes the reverted `verification/srp_drag_decrement/` pinned-envelope harness (see `reverted/README.md`)'. `ls verification/` returns 13 example directories and none is srp_drag_decrement; `ls reverted/` returns README.md and srp_drag_decrement/. The path is stale. Severity info is right — the same sentence points at reverted/README.md, so a reader recovers in one hop.

> Evidence re-read: deep_causality_cfd/studies/README.md:56-60; `ls deep_causality_cfd/verification/` (13 dirs, no srp_drag_decrement); `ls deep_causality_cfd/reverted/` (README.md, srp_drag_decrement)

---
