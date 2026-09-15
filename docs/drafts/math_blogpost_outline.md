# Unified Math 

---

## Part 1: Raw information, by source

Sources:

- **(A)** `deep_causality_unified_math/README.md`
- **(B)** `website/docs/src/content/docs/concepts/uniform-math.md`
- **(C)** `examples/mathematics_examples/README.md`
- **(D)** `examples/mathematics_examples/2_composition/README.md`
- **(E)** Aristotle, *Nicomachean Ethics* I.1, "of medicine, the end is health; of shipbuilding, a ship; of generalship, victory"
- **(F)** `docs/writing_guides/Aristotelian rules of communication.pdf`: ethos, pathos, logos
- **(G)** `openspec/notes/archive/unified_math/unified_math_gaps.md` and `HKT-LAW-FINDINGS.md`
- **(H)** Common practice in scientific programming; the author's assessment, to be stated as such in the post
- **(I)** Published evidence for the premise, numbered I1 to I13 below; every pain-point row cites at least one

Published references (I):

- **I1** Hatton, L. (1997). The T-experiments: errors in scientific software. *IEEE Computational Science & Engineering* 4(2), 27–38. https://www.leshatton.org/Documents/Texp_ICSE297.pdf. Reports experiment T2, the seismic study also published as Hatton & Roberts (1994), *IEEE TSE* 20(10), 785–797, DOI 10.1109/32.328993 (full text paywalled; cite the 1997 paper).
- **I2** Miller, G. (2006). A scientist's nightmare: software problem leads to five retractions. *Science* 314(5807), 1856–1857. https://www.science.org/doi/full/10.1126/science.314.5807.1856
- **I3** Merali, Z. (2010). Computational science: ...Error. *Nature* 467, 775–777. https://www.nature.com/articles/467775a
- **I4** Hannay, J. E., MacLeod, C., Singer, J., Langtangen, H. P., Pfahl, D., Wilson, G. (2009). How do scientists develop and use scientific software? *ICSE SECSE Workshop*. https://dl.acm.org/doi/10.1109/SECSE.2009.5069155
- **I5** Wilson, G. et al. (2014). Best practices for scientific computing. *PLOS Biology* 12(1), e1001745. https://journals.plos.org/plosbiology/article?id=10.1371/journal.pbio.1001745
- **I6** Kelly, D., Sanders, R. (2008). Assessing the quality of scientific software. *First International Workshop on Software Engineering for Computational Science and Engineering*, Leipzig. http://se4science.org/workshops/secse08/Papers/Kelly.pdf
- **I7** Storer, T. (2017). Bridging the chasm: a survey of software engineering practice in scientific programming. *ACM Computing Surveys* 50(4), 47:1–47:32. https://dl.acm.org/doi/abs/10.1145/3084225
- **I8** Consortium for Python Data API Standards. Python array API standard, Purpose and scope. https://data-apis.org/array-api/2024.12/purpose_and_scope.html
- **I9** Sommer, H., Gilitschenski, I., Bloesch, M., Weiss, S., Siegwart, R., Nieto, J. (2018). Why and how to avoid the flipped quaternion multiplication. *Aerospace* 5(3), 72. https://arxiv.org/abs/1801.07478
- **I10** Mars Climate Orbiter Mishap Investigation Board (1999). Phase I Report. NASA. https://llis.nasa.gov/llis_lib/pdf/1009464main1_0641-mr.pdf
- **I11** Lions, J.-L. et al. (1996). Ariane 5 Flight 501 Failure: Report by the Inquiry Board. ESA/CNES. http://sunnyday.mit.edu/nasa-class/Ariane5-report.html
- **I12** McCullough, B. D. (1998). Assessing the reliability of statistical software: Part I. *The American Statistician* 52(4), 358–366. https://www.tandfonline.com/doi/abs/10.1080/00031305.1998.10480597. Also Keeling, K. B., Pavur, R. J. (2007). A comparative study of the reliability of nine statistical software packages. *Computational Statistics & Data Analysis* 51(8), 3811–3831. https://www.sciencedirect.com/science/article/abs/pii/S0167947306000491
- **I13** Váňa, F., Düben, P., Lang, S., Palmer, T., Leutbecher, M., Salmond, D., Carver, G. (2017). Single precision in weather forecasting models: an evaluation with the IFS. *Monthly Weather Review* 145(2), 495–502. https://journals.ametsoc.org/view/journals/mwre/145/2/mwr-d-16-0228.1.xml. Also Higham, N. J., Mary, T. (2022). Mixed precision algorithms in numerical linear algebra. *Acta Numerica* 31, 347–414. https://eprints.maths.manchester.ac.uk/2849/

### The pain points: what goes wrong when math domains cross

Eight pain points, each a translation done by hand at a boundary. The table names the symptom the
reader recognises, the root the post will argue, and the section that answers it.

| # | Pain point | Symptom the reader knows | Root | Answered in |
|---|---|---|---|---|
| P1 | Container translation | Every library owns its own array; a crossing copies, reorders, and reindexes. Row-major against column-major, zero-based against one-based, dense against sparse. Half the code is plumbing (B, H, I1, I8) | No operation vocabulary shared across containers | §2 Ontology |
| P2 | Convention drift | Sign conventions, FFT normalisation, blade order, quaternion handedness, boundary orientation differ per library and fail silently. A wrong sign passes the tests (B, H, I2, I3, I9, I10) | Each library defines its conventions for itself | §2, §3 |
| P3 | Precision lock-in | One library computes in `f64`, a neighbour in `f32`, and the pipeline downcasts without a warning. A drift of `1e-13` appears with no owner, and there is no way to rerun the whole chain at a wider precision to find it (A, C, H, I1, I13) | Precision baked into every call site | §3, §4 |
| P4 | Unchecked laws | An elimination assumes a field and is handed integers; a solver assumes symmetry; a "monad" reshapes its input. Nothing refuses; the violation surfaces as a wrong number (B, G, H, I1, I6, I11) | Laws live in documentation, never in types | §3 Metaphysics |
| P5 | Duplicated primitives | Each domain library ships its own variance, its own entropy, its own random stream, and the definitions disagree: sample against population, log base, seeding. One quantity, two numbers (B, H, I12) | No shared lower tier | §2, §5 |
| P6 | Structure loss at the boundary | A field of rotors flattens into a float buffer to cross into the next library, which parses it back. The derivation on paper and the code stop matching, and a reviewer can check only the numbers (A, B, H, I6) | Containers of one library cannot hold values of another | §2, §3 |
| P7 | One precision for everyone | The library author chose a scalar once, for every user. The program pays `f64` memory on parts that are noise-bound and loses digits on parts that are reference-bound, and the bill arrives as a memory rung and a wall clock (A, H, I13) | Precision is a library property instead of a program parameter | §5, §6 |
| P8 | Failure that flows | A `NaN` in step two reaches step five and is averaged into the answer; the log says nothing about where it entered (B, H, I6) | No shared step-composition law with short-circuit | §2, §6 |

Two smaller items, mentioned once each: external build weight (BLAS, LAPACK, FFTW, per platform)
against a stack with no required external dependency (A); and the composed pipeline that no single
library's test suite covers, answered by residuals measured at the composed level (A).

What stays unsolved, to be stated in the close: crossings at the outer edge of the stack remain
translations (a `BFloat16` slice matching an accelerator's `bf16` buffer is the one bridge kept on
purpose) (A); the witness table's `none` row (A); `Float106` costs two to four times an `f64`
operation (A).

### Evidence for the premise, by pain point

The figures and quotations below are what the post may cite. Each is checked against the source
named; a figure the post cannot trace to one of these lines does not go in.

- **Boundaries are where faults cluster (P1).** Hatton's static study of 3.3 million lines of Fortran and 1.9 million of C, from 47 and 26 organisations, found interface inconsistencies "at the rate of one in every 7 interfaces on average in Fortran, and one in every 37 interfaces in C" (I1). The array API consortium states the modern form: the APIs of NumPy, TensorFlow, PyTorch, JAX, Dask, CuPy and MXNet "are largely similar, but with enough differences that it's quite difficult to write code that works with multiple (or all) of these libraries" (I8).
- **A convention fails silently and the tests pass (P2).** An in-house data-reduction program flipped the sign of anomalous differences, the structures came out in the wrong hand, and five papers were retracted, three of them from *Science* (I2). Merali's account: "a computer program supplied by another lab had flipped a minus sign, which in turn reversed two columns" (I3). Two quaternion multiplications remain "in common use", Hamilton's and the flipped form associated with JPL, and formulas migrate between them with a recipe (I9). The Mars Climate Orbiter was lost to thruster data supplied in pound-seconds where the receiving software required newton-seconds; the board's root cause is the failed translation of units "in a segment of ground-based, navigation-related mission software" (I10).
- **Drift arrives without an owner (P3).** In experiment T2, independent implementations of the same seismic algorithms on the same input "gradually degenerated from 6 significant figures to 1 significant figure during the computation", and Hatton lays the disagreement "squarely at the door of software failure" after rejecting other causes (I1). Hatton's conclusion: results of calculations involving significant software "should be treated with the same measure of disbelief as an unconfirmed physical experiment" (I1).
- **A law assumed in documentation fails at run time (P4).** Ariane 5 flight 501: "a data conversion from a 64-bit floating-point number to a 16-bit signed integer value" overflowed, the Operand Error halted the inertial reference system, and "not all the conversions were protected because a maximum workload target of 80% had been set" (I11). Hatton names the belief directly: "It is therefore a common fallacy, that if something compiles, it is OK, apart from errors of the mind" (I1). Kelly and Sanders on why one passing test proves little: computational software "represents continuous models using finite resources", so "one set of test data that succeeds does not guarantee the success of test data anywhere in its neighbourhood" (I6).
- **Two libraries, two numbers for one quantity (P5).** McCullough's benchmark method for estimation, random number generation and distributions found packages disagreeing on the same problems (I12); Keeling and Pavur applied it to nine packages and found that the choice of package and of default settings changes the answer (I12).
- **The software goes invisible behind the model (P6).** Kelly and Sanders: scientists "assess their models, not the software"; "the software essentially goes invisible", and one interviewee's software engineer was told to "keep his hands off my model" (I6). Testing is against "limited oracle data" from the science domain, so the composed pipeline has no oracle of its own (I6).
- **One precision for everyone has a price (P7).** ECMWF ran its forecast model at single precision with "no noticeable reduction in accuracy, and an average gain in computational efficiency by approximately 40%" (I13). Higham and Mary survey the alternative, "adapting the precision to the data" per algorithm stage (I13).
- **A wrong number that keeps flowing is the worst outcome (P8).** Kelly and Sanders' interviewee: the software "had better not lie to him. Much preferred is a complete crash of the system than an insidious error that goes undetected and provides data that corrupts the insight" (I6).
- **The background, for the opening paragraph.** Scientists spend "30% or more of their time developing software" (I5). Of nearly two thousand surveyed, most learned to program from peers and self-study, and while many rate testing important, fewer believe they understand it (I4). Storer calls the gap between software engineering and scientific programming "a serious risk to reliable scientific results" (I7). Hatton adds the cultural cause: "scientists tend to swap code rather than relying on the independent verification they pursue naturally for an experimental result" (I1).

### Source (J): the code on `main`, read 2026-09-14

| Fact the post uses | Where it lives |
|---|---|
| `Lift` trait, `lift` / `lift_count` / `lower` / `to_count`, each with a `try_` form; the "Why not `as`, and why not `From`" argument | `deep_causality_num/src/lift/mod.rs` |
| `Real` is the analytic axis without field invertibility, so `Dual` qualifies; `f32`, `f64`, `BFloat16`, `Float106` all implement it | `deep_causality_algebra/src/algebra/real.rs` |
| `Scalar: Real + Div + FromPrimitive`, blanket-implemented; `calculus` writes its operators against it, so one model evaluates at `f64` for the value and at `Dual` for the derivative | `deep_causality_algebra/src/algebra/scalar.rs` |
| `LogBase`, `ZeroPolicy`, `Normalisation`, `EntropyConfig`: the three axes the workspace's own entropy implementations disagreed on, made parameters | `deep_causality_stats/src/types/{log_base,zero_policy,normalisation,entropy_config}/mod.rs` |
| A chain complex mentions no space, metric or cell; a quantum code is one with no cells; `β₁` of real projective space is 0 over ℚ and 1 over 𝔽₂; `∂∘∂ = 0` "is not checkable by the trait" | `deep_causality_homology/src/lib.rs` |
| One crate owns `Cl(p, q, r)`, the east and west coast conventions, `detect_convention`, `east_to_west`, `west_to_east`; zero dependencies | `deep_causality_metric/src/lib.rs` |
| `BFloat16`'s double rounding is harmless for `+ − × ÷ sqrt` because `24 ≥ 2·8 + 2`, citing Figueroa (1995); `round_from_f64` goes through round-to-odd, citing Boldo and Melquiond (2008) | `deep_causality_num/src/float_bfloat16/mod.rs` |
| `Float106` is the unevaluated sum of two `f64`, ~106-bit significand, roughly 2 to 4× the cost of an `f64` operation | `deep_causality_num/src/float_106/mod.rs` |
| `CommutatorTolerance` sets `unit_roundoff: R::epsilon()`, safety factor 8, `γ_n = n·u/(1 − n·u)`: the acceptance gate is a function of the working type | `deep_causality_quantum/src/types/qcm/markov_freeze.rs` |
| "Every tolerance in the run derives from its `epsilon()`; switch it to `f32`, `f64`, or `Float106` and the thresholds move with it" | `examples/quantum_examples/qcl_examples/qcl_qcm_freeze/main.rs:30-34` |
| `StandardUniform` claimed uniform reals, machine words and Booleans at once; coherence (E0119) refused the blanket implementation over the algebra tower; the type split three ways | commit `a70d2ffd3` |
| 41 HKT witness types across the workspace, test fixtures excluded; 13 of them in `topology` alone | `grep -rn "impl.*HKT for" deep_causality_unified_math/*/src deep_causality_core/src deep_causality/src \| grep -v utils_tests` |
| 196 property statements mapped Lean to Rust, 192 proved; CI fails when an id lacks either side | `lean/THEOREM_MAP.md`, `.github/workflows/formalization.yml` |
| The stack has no required external dependency; four crates have an optional one behind a default-off feature | `deep_causality_unified_math/README.md`, Cargo manifests |

Two discrepancies found while checking, to be fixed in the tree rather than written around:

- `deep_causality_num/src/lift/mod.rs` says "the three shipped scalars" in its
  "Why not `as`, and why not `From`" section; there are four since `BFloat16` landed.
- `AGENTS.md` lists `deep_causality_multivector` and `deep_causality_topology` as `unsafe_code`
  exemptions. Neither crate contains `unsafe` any more and both carry `[lints] workspace = true`.

---

## Aristotelian Framework

### Ethos (Credibility) — built by making the code the citation, and by naming this project's own failures first

**The primary move: every claim carries a path.** The reader of a post about mathematics software
has been promised coherence before. What separates this from the promise is that each section
names the file, the commit or the test that settles it, and invites the reader to open it. State
that contract in the first hundred words and keep it: no figure appears that is not traceable to
source (A) through (J).

**Authority through the crossing, not the lane.** Four fields meet here and the post moves through
all of them with precision: numerical analysis (Figueroa's double-rounding bound, Boldo and
Melquiond's round-to-odd, Higham's `γ_n` forward-error model), applied category theory (the GAT
witness standing in for a type constructor Rust cannot name), Rust's type system (coherence
refusing a blanket implementation, E0119, and what that refusal proved about the ontology), and
cloud economics (instance rungs, list prices, bytes moved). Most writing on scientific software
holds one lane. Let the reader feel the reading behind all four.

**The code cites its own theorems.** `BFloat16`'s rounding kernel carries the two papers that
justify it in the docstring, in the file, next to the arithmetic. That is a small fact and it does
more for credibility than any adjective: it shows a codebase where the burden of proof was
accepted rather than asserted (J).

**Verification that can fail.** 192 of 196 property statements are proved in Lean and bound by id
to a Rust witness, and CI fails when either side is missing (J). Say the number, say the four that
are not proved, and say what the binding does not do: there is no tool that converts a Lean proof
into a Rust test, so the statement is transcribed once per layer and the map is the bridge.

**Intellectual honesty moment 1 (Paragraph 5) — turn the knife inward first.** The strongest
credibility in the whole post is the `LogBase` docstring: *this workspace's own* entropy
implementations disagreed, the causal-discovery paths computing in bits and the thermodynamics
kernel in nats, "which is a different number, not a rounding" (J). Do not present the parameter as
foresight. Present it as a bug that was found, named, and typed. A reader who watches an author
convict himself will believe the rest.

**Intellectual honesty moment 2 (Paragraph 6) — name where the type system stops.** The homology
crate states that `∂ₖ ∘ ∂ₖ₊₁ = 0` "is not checkable by the trait, and every Betti number this crate
computes is wrong without it" (J). Quote it. An author who marks the edge of his own guarantee is
trusted inside it.

**Intellectual honesty moment 3 (Paragraph 9) — label the estimate as an estimate.** The economic
section is arithmetic on stated assumptions and published list prices, and one number inside it is
transferred from ECMWF's measurement on a different model and a different machine (A, I13). Say
both before the table, not after. The 24 % on the hourly rate is arithmetic the reader can redo;
the 32 % on the clock is borrowed evidence. Different standing, stated as different.

**Two things the post declines to claim.** It does not claim that no one else has done this, and
it does not claim the boundary problem is solved. Crossings at the outer edge of the stack are
still translations; the `none` row of the trait table is still open work; `Float106` still costs
two to four times an `f64` operation (A). Put all three in the close.

### Pathos (Emotional connection) — the reader is a scientist who has been lied to by a number

The audience is not moved by architecture. They are moved by the memory of a number they trusted.
Three moments carry the post, and everything between them is quiet.

**Paragraph 2 — the retraction.** This is the first emotional peak and it must be told as a story,
not cited as a fact. A researcher's in-house data-reduction program flipped the sign of anomalous
differences. The structures came out in the wrong hand. Five papers were retracted, three of them
from *Science* (I2, I3). Now land the point the reader has been feeling: the program did not crash,
the tests did not fail, and the numbers looked exactly as good as correct numbers look. Miller's
title says the rest, and it is worth quoting as a title: *A scientist's nightmare*.

**Paragraph 3 — the number that quietly stopped being true.** Hatton's experiment T2 ran
independent implementations of the same seismic algorithms on the same input, and agreement
"gradually degenerated from 6 significant figures to 1 significant figure during the computation"
(I1). Deliver that as a sentence on its own line. Then let Hatton say the uncomfortable part:
results from significant software "should be treated with the same measure of disbelief as an
unconfirmed physical experiment" (I1). Pause there. Do not soften it, do not answer it yet.

**Paragraph 3 — the interviewee, as the emotional anchor.** Kelly and Sanders' scientist on what he
actually fears: the software "had better not lie to him. Much preferred is a complete crash of the
system than an insidious error that goes undetected and provides data that corrupts the insight"
(I6). That is the whole of Point 1 in one person's voice. Give it its own paragraph break.

**Paragraph 5 — the confession, delivered flatly.** Pathos here is inverted: no drama, no hedge,
just the admission that the same disagreement the post has spent three paragraphs describing was
found inside this codebase, between its own crates, in bits against nats (J). Understatement is the
instrument. The reader's trust moves the moment the author stops being the hero of his own post.

**Paragraph 9 — the bill.** The reader who has waited three days for a queue slot, or watched a
grant line disappear into node-hours, does not need persuading that memory has a price. Show the
two rows and let the subtraction do the work: `$22.62` against `$11.72`, same cores, same answer
(A). Do not editorialise on it. The number is the emotion.

**Closing — quiet.** The close should drop, not build. Same register as the `05_metadata_problem`
ending: the power is in the understatement.

### Logos (Reason) — one deduction, stated once, then instantiated three times

**Premise.** Errors and costs in cross-domain scientific software cluster at the boundaries
between mathematical domains. Hatton measured the error side, one interface inconsistency per
seven in Fortran and per thirty-seven in C, across 5.2 million lines from 73 organisations (I1).
ECMWF measured the cost side, a 40 % efficiency gain from one precision decision the library
author had already made for everyone (I13).

**Deduction.** A boundary exists because each library privately owns four things: its container,
its conventions, its primitives, and its scalar. Making any of the four shared removes that
boundary rather than guarding it. A shared operation vocabulary (the witness) removes the
container boundary; a shared lower tier (`metric`, `stats`, `num`) removes the convention and
primitive boundaries; a scalar left as a type parameter removes the precision boundary.

**Conclusion.** With the boundary gone, precision stops being a property the program inherits and
becomes a parameter the program sets, per part, against a stated requirement. A parameter has a
price, and the price can be read off a table.

**The structural spine.** Each paragraph closes on the premise the next one opens with. Track it:
¶1 ends on "where should a translation live"; ¶4 answers "one level down, in the type"; ¶6 ends on
"the tower is generic in its scalar and nothing named a scalar"; ¶7 opens there. A reader who
skims the first and last sentence of every paragraph gets the complete argument.

**The counting argument, stated as arithmetic, once (¶4).** With `K` container types, pairwise
bridges number up to `K(K − 1)`, one per ordered pair, each written and tested where it is used.
Witnesses number `K`, one per container, tested once at the witness. The workspace carries 41
witnesses; the pairwise alternative at that count is 1640 possible bridges (J). Hatton's rate
applies to bridges, so fewer bridges means fewer places for the rate to act (I1). State the
arithmetic, show the witness in eight lines, and stop. This is the author's own assessment (H) and
is labelled as such.

**Where the argument is deliberately left open.** The post does not argue that the categorical
layer is complete. The trait table's `none` row is shown as the work list, and two witnesses that
violated monad laws are named along with what was done about them: one was fixed, one gave up
`Monad` (A, G). A law table with holes in it is evidence that the table means something.

---

## Part 2: The nine paragraphs

Three points, three paragraphs each. Estimated 3,000 to 3,400 words, four code blocks, four
tables. Every section opens on what the conventional way costs, shows the unified way in code that
exists, and names the pain-point rows it answers.

---

## Point 1 — The pain: every boundary is a translation written by hand

### Paragraph 1 — The afternoon that went to plumbing (Topic)

**Main point.** A simulation that crosses two branches of mathematics pays at every boundary, and
the payments share one shape: a translation the programmer wrote by hand, at a call site, where
nothing could check it.

**Supporting points:**

- The scene the reader recognises: a mesh walk in one library, a contraction in a second, a rotor
  in a third; row-major against column-major, zero-based against one-based, dense against sparse.
  Half the code is plumbing (B, H, P1).
- The consortium behind the Python array API standard states the modern form plainly: the APIs of
  NumPy, TensorFlow, PyTorch, JAX, Dask, CuPy and MXNet "are largely similar, but with enough
  differences that it's quite difficult to write code that works with multiple (or all) of these
  libraries" (I8).
- Scientists spend "30% or more of their time developing software" (I5); most learned to program
  from peers and self-study (I4). Storer calls the resulting gap "a serious risk to reliable
  scientific results" (I7).
- Hatton's static analysis puts a rate on the boundaries themselves: one interface inconsistency
  per seven in Fortran and per thirty-seven in C, over 3.3 million lines of Fortran and 1.9 million
  of C, from 47 and 26 organisations (I1).

**Connectors:** "for instance" (the scene), "consequently" (plumbing displaces the science),
"in particular" (Hatton's rate lands on the interfaces, not the algorithms).

**Hook direction:** The mathematics is checked, the derivation is checked, and the paper is
checked. The sentence between two libraries is checked by nobody.

**Closing sentence direction:** Every one of those sentences was written by hand, and a
hand-written sentence has no owner when it fails.

### Paragraph 2 — The translation that passes every test (The silent failure)

**Main point.** A convention is not a value the program holds, so the program cannot check it; when
a convention is translated wrongly the result is a plausible number rather than an error.

**Supporting points:**

- The retraction, told as a narrative: an in-house data-reduction program flipped the sign of
  anomalous differences, the protein structures came out in the wrong hand, and five papers were
  retracted, three from *Science* (I2). Merali's account of the mechanism: "a computer program
  supplied by another lab had flipped a minus sign, which in turn reversed two columns" (I3).
- This is not one accident but a standing condition. Two quaternion multiplications, Hamilton's
  and the flipped form associated with JPL, both remain "in common use", and formulas migrate
  between them with a recipe (I9).
- The Mars Climate Orbiter was lost to thruster data supplied in pound-seconds where the receiving
  software required newton-seconds; the board located the root cause in "a segment of ground-based,
  navigation-related mission software" (I10).
- Hatton names the belief that lets this survive review: "It is therefore a common fallacy, that if
  something compiles, it is OK, apart from errors of the mind" (I1).
- Why one passing test proves nothing here: computational software "represents continuous models
  using finite resources", so "one set of test data that succeeds does not guarantee the success of
  test data anywhere in its neighbourhood" (I6).

**Connectors:** "moreover" (from the single retraction to the standing condition), "similarly"
(Mars Climate Orbiter), "because" (why the test suite is blind to it).

**Rows answered:** P2, P4.

**Closing sentence direction:** The program did not crash, the suite did not fail, and the numbers
looked exactly as good as correct numbers look.

### Paragraph 3 — The drift with no owner and the cost with no line item (The stakes)

**Main point.** A hand-written translation fails without an owner and costs without a line item,
and the published record has measured both.

**Supporting points:**

- Hatton's experiment T2: independent implementations of the same seismic algorithms, on the same
  input, agreed to six significant figures at the start and "gradually degenerated from 6
  significant figures to 1 significant figure during the computation"; he lays the disagreement
  "squarely at the door of software failure" after rejecting other causes (I1).
- His verdict, quoted whole: results of calculations involving significant software "should be
  treated with the same measure of disbelief as an unconfirmed physical experiment" (I1).
- Why nobody owns it: scientists "assess their models, not the software"; "the software essentially
  goes invisible", and one interviewee's software engineer was told to "keep his hands off my
  model" (I6). Testing runs against "limited oracle data" from the science domain, so the composed
  pipeline has no oracle at all (I6).
- The cost side, measured: ECMWF ran its operational forecast at single precision with "no
  noticeable reduction in accuracy, and an average gain in computational efficiency by
  approximately 40%" (I13). That 40 % was available the whole time and was spent on a decision
  made inside a library, once, for everyone.
- Two libraries, one quantity, two answers: McCullough's benchmark method found statistical
  packages disagreeing on the same problems, and Keeling and Pavur found across nine packages that
  the choice of package and of default settings changes the answer (I12).
- The interviewee's own statement of the stakes: the software "had better not lie to him. Much
  preferred is a complete crash of the system than an insidious error that goes undetected and
  provides data that corrupts the insight" (I6).
- Show the pain-point table here, trimmed to two columns: symptom and root. Eight rows, five roots:
  a container, a convention, a scalar, a law, a primitive (H).

**Connectors:** "because" (invisible software, unowned drift), "on the other hand" (pivot from
error to cost), "consequently" (five roots).

**Rows answered:** P1, P3, P5, P6, P7, P8 named in the table.

**Closing sentence direction, and the transition into Point 2:** Eight rows reduce to one question.
If the call site is the wrong place for a translation, where should it live?

---

## Point 2 — The inversion: move the translation into the type

### Paragraph 4 — One witness per container, instead of one bridge per pair (The mechanism)

**Main point.** The translation moves one level down, from the call site into the container's own
declaration, and the count of things that can be wrong falls from quadratic to linear.

**Supporting points:**

- Rust has no native higher-kinded types, so a witness stands in: a zero-sized type whose generic
  associated type projects back to the container. The whole of it for tensors:
  `impl HKT for CausalTensorWitness { type Type<T> = CausalTensor<T>; }` (J, B).
- A crate that owns a container declares its witness once and implements the categorical traits
  against it. Two mechanisms fall out. **Nesting:** a witness accepts any element type, including
  one another crate owns, so `CausalTensor<CausalMultiVector<FloatType>>` is an ordinary tensor and
  one `fmap` turns every cell by a rotor. **Closure reach:** `extend` hands a cursor to a closure,
  and the closure may call into any crate it likes (A).
- The call site after the inversion, in full, as the first code block:
  `CausalTensorWitness::fmap(field, |v| rotor.geometric_product(&v).geometric_product(&rotor_rev))` (A).
- The counting argument, labelled as the author's assessment (H): `K` containers admit up to
  `K(K − 1)` ordered pairwise bridges, each written and tested where it is used; witnesses number
  `K`, tested once. The workspace carries 41 witnesses, 13 of them in `topology` alone (J).
- Hatton's rate applies to bridges (I1). Fewer bridges, fewer places for that rate to act.
- The same trait name means the same operation on every container: `fmap`, `bind`, `extend` and
  `extract` read identically on a tensor, a sparse matrix, a multivector, a manifold and a
  propagating effect (B). Show the witness table; read down a column for the transfer claim.

**Connectors:** "that is" (reformulating the witness), "consequently" (the arithmetic),
"in addition" (nesting and closure reach).

**Rows answered:** P1, P6.

**Closing sentence direction:** The container boundary is the easy one. The conventions are harder,
because a convention is not a container; it is an agreement.

### Paragraph 5 — Cutting at the mathematical joint, and the disagreement found at home (The ontology)

**Main point.** A convention stops being an agreement when one crate owns it as a type, and the
crates are cut where the mathematics divides rather than where the application does.

**Supporting points:**

- `deep_causality_metric` owns `Cl(p, q, r)`, the east coast `(−+++)` and west coast `(+−−−)`
  conventions as distinct types, `detect_convention`, and the two conversions between them. It has
  zero dependencies and sits at tier 0, so `Cl(3,1)` in `multivector` and the Einstein tensor in
  `tensor` name the same signature by construction (J).
- The homology crate is the clearest case of cutting at the mathematical joint. A chain complex is
  a sequence of modules with `∂ₖ ∘ ∂ₖ₊₁ = 0`, and "that definition mentions no space, no metric and
  no cell". A quantum error-correcting code is a chain complex with no cells: `H_X` and `H_Z` are
  parity-check matrices whose product vanishes over 𝔽₂ (J). The crate's own sentence is the thesis
  of this paragraph: "A crate that wanted homology should not have to carry a Hodge star to get it."
- **The honesty moment, and the emotional centre of Point 2.** The `LogBase` docstring, quoted
  whole: the base "is a parameter because the workspace's shipped entropy implementations disagree
  on it: the causal-discovery paths compute in bits, the thermodynamics kernel in nats. The two
  differ by a factor of `ln 2`, which is a different number, not a rounding" (J). The same is true
  of the other two axes, and `ZeroPolicy` and `Normalisation` each carry the same admission. State
  it as what it is: P5 was found inside this workspace, between its own crates, and the fix was to
  make the disagreement a type rather than a default (J).
- `EntropyConfig` travels as one value because the three axes are independent and the shipped
  implementations disagreed on all three at once. Its docstring adds the discipline that keeps a
  configuration type honest: "Each combination named here has a caller; none is speculative" (J).
- The ontology shows up again in the random-number crate. `StandardUniform` claimed uniform reals,
  machine words and Booleans at once, and the coherence checker refused the blanket implementation
  over the algebra tower, because it "cannot prove `u64` will never be a real field". The type split
  three ways, and five internal sites that had been drawing a machine word through the uniform-real
  sampler now say what they always meant (J, commit `a70d2ffd3`).
- Read that last item for what it is: the compiler located an ontological confusion that no test
  had objected to.

**Connectors:** "for instance" (metric), "to illustrate" (homology and the quantum code),
"in fact" (the entropy confession), "similarly" (the sampler split).

**Rows answered:** P2, P5.

**Closing sentence direction:** A convention that lives in a type can be converted. A law that lives
in a docstring cannot be enforced, and the next question is which of the two the guarantees live in.

### Paragraph 6 — Laws that are enforced, withdrawn, or admitted (The guarantee and its edge)

**Main point.** The categorical traits carry laws, the laws are tested at the witness, and where a
law cannot hold the trait is withdrawn rather than shipped; where the type system cannot reach, the
code says so.

**Supporting points:**

- Two real defects were found by law tests rather than by reading: `CsrMatrixWitness::bind` rebuilt
  every matrix as `1 × count`, and `CausalTensorWitness` violated monad right identity, returning a
  `[2, 3]` as a `[6]`. Both are fixed; `bind` now keeps the input's shape when the map is
  shape-preserving (A, G).
- Where the law cannot hold, the trait goes. `CausalMultiVectorWitness` gave up `Monad`, because no
  metric choice satisfies both identity laws; the shaped `linear` witnesses stop at `Applicative`
  (A, B, G).
- The witness docstring records the corner it cannot close: a one-element tensor can carry shape
  `[]`, `[1]` or `[1, 1]`, `bind` must choose, right identity wins, and associativity parts company
  on that input. The note says so in the file (J).
- Above the trait laws sits the formal layer: 196 property statements mapped between Lean and Rust,
  192 proved, each carrying the same id on both sides, and CI failing when an id lacks either side.
  No tool converts a Lean proof into a Rust test; the statement is transcribed once per layer and
  the map is the bridge (J).
- **The honesty moment.** The homology crate states the edge of its own guarantee: every implementor
  of `ChainComplex` owes `∂ₖ ∘ ∂ₖ₊₁ = 0`, and "it is not checkable by the trait, and every Betti
  number this crate computes is wrong without it" (J). Quote it exactly.
- The trait table's `none` row is the work list, published rather than hidden:
  `NaturalTransformation`, `Category`, `Kleisli`, `Bifunctor` and `Profunctor` have no implementers
  outside `haft` (A).

**Connectors:** "for instance" (the two defects), "conversely" (the withdrawn traits),
"more importantly" (the Lean layer), "yet" (where the type system stops).

**Rows answered:** P4, P8.

**Closing sentence direction, and the transition into Point 3:** Every crate above `num` is generic
in its element, bounded by `Real`, `RealField` or `Scalar`, and across the whole tower nothing ever
named a concrete floating-point type. That omission is the third point.

---

## Point 3 — Precision as a parameter, and what it costs

### Paragraph 7 — One alias, four arithmetics (The mechanism)

**Main point.** Because no crate above `num` named a concrete scalar, a program names its working
type once and the arithmetic of the whole program follows the name.

**Supporting points:**

- `type FloatType = f64;` is the entire mechanism. Switch the alias and the arithmetic changes
  precision; nothing else moves, provided nothing else was ever spelled `f64` (A).
- The four shipped real fields, as the first table of this point: `BFloat16` at 2 bytes and 2
  decimal digits, `f32` at 4 and 7, `f64` at 8 and 16, `Float106` at 16 and 31. Precision is the
  significand and range is the exponent, and the two move independently: `BFloat16` holds every
  magnitude `f32` holds and resolves fewer of them (A).
- Two of the four are built in `deep_causality_num` and reach the tower through the same `Float`
  blanket implementations as the hardware types, so nothing in the tower knows the difference (A,
  J). `Float106` is the unevaluated sum of two `f64`, and the type's own docstring prices it at two
  to four times an `f64` operation (J).
- `BFloat16`'s docstring is the ethos beat: it proves its double rounding harmless for `+`, `−`,
  `×`, `÷` and `sqrt` by Figueroa's `2p + 2` bound at `p = 8`, and routes `round_from_f64` through
  round-to-odd citing Boldo and Melquiond, because two roundings in a row can land a value exactly
  on a tie it was never on (J). Mention it in one sentence and move on; the restraint is the point.
- Three kinds of number must still cross the alias boundary: a configuration literal (`f64` in the
  source, the widest a source file holds), a count (`u64` or `usize`), and a result on its way to
  `println!`. The two obvious spellings each fail on one shipped scalar: `x as FloatType` casts
  between primitives only and stops compiling the day the alias becomes `Float106`, and
  `FloatType::from(0.5)` fails for `f32`, which has no `From<f64>`. The `lift` module writes the
  crossings once over `FromPrimitive` and `ToPrimitive`, which all four implement (A, J).
- The measurement, as the second code block and table: the telescoping series, a million terms,
  against its closed form. `BFloat16` earns 2 correct digits, `f32` 4, `f64` 13, `Float106` 30. One
  program, four runs, one alias changed (A).
- Read the failures, because they are instructive rather than embarrassing. `BFloat16` stops adding
  at `k = 23`: 999 978 of the million terms are absorbed into a sum that stalls at `0.96875`. `f32`
  stalls past `k = 4000`. Both are the same mechanism at different scales (A).

**Connectors:** "for instance" (the four types), "however" (why `as` and `From` both fail),
"as shown by" (the series table).

**Rows answered:** P3.

**Closing sentence direction:** Changing one alias changed thirty digits of the answer. The next
question is how far that reaches: through four crates, and into the thresholds that decide whether
a result is accepted.

### Paragraph 8 — How far the parameter reaches (The generalisation)

**Main point.** The parameter reaches past the arithmetic: it carries through a composition of four
crates with nothing converted between them, it re-derives the tolerances that decide acceptance,
and precision is one instance of a pattern that also covers the coefficient field and the scalar
itself.

**Supporting points:**

- Composition, as the third code block: one field sampled into a `CausalTensor`, placed on a line
  manifold, differentiated by comonadic extension, paired into `Cl(2,0)` vectors, and turned
  through a thousand quarter turns by one rotor pair. Two identities hold exactly, so the residual
  is the rounding of the whole pipeline at one precision (A).
- The result table, three rows: `Σ Δφ` at `2.0e-7`, `6.9e-17` and `1.7e-32`; the thousand geometric
  products at `1.5e-7`, `1.6e-13` and `1.3e-29`. Twenty-five orders of magnitude from `f32` to
  `Float106`, and the sentence that earns the section: "Nothing was converted between crates,
  because there was nothing to convert" (A).
- The capstone, in one line: four crates parallel-transporting a unit timelike spinor along a
  discretized Minkowski worldline in `Cl(3,1)`, drifting about `1.7e-31` from the closed-form
  `(cosh θ, sinh θ)` at `Float106` (A, D).
- **The reach nobody expects, and the strongest single fact in the post.** The parameter moves the
  acceptance gates with it. `CommutatorTolerance` sets `unit_roundoff: R::epsilon()` and builds its
  forward-error budget as `γ_n = n·u/(1 − n·u)` with a safety factor of 8 (J). The example's own
  docstring states the consequence: "Every tolerance in the run derives from its `epsilon()`;
  switch it to `f32`, `f64`, or `Float106` and the thresholds move with it" (J). State the
  mechanism and stop: the threshold is computed from the type, so the two move together, and a
  threshold written as a literal does not.
- Precision is one axis of a general pattern. The coefficient field is another, and it changes the
  answer rather than its accuracy: real projective space has `β₁ = 0` over ℚ and `β₁ = 1` over 𝔽₂,
  so `HomologyField` carries the choice at the call site (J).
- The scalar itself is the third. `Real` is the analytic axis decoupled from field invertibility,
  so `Dual` qualifies as a `Scalar` even though `ε` is a zero divisor, and `deep_causality_calculus`
  writes its operators against `Scalar`. One model therefore evaluates at `f64` for the value and at
  `Dual` for the derivative, and `Dual<Dual<…>>` nests for higher derivatives (J).
- The examples are written to this discipline: each keeps its alias in `main.rs`, none carries a
  conversion helper of its own, and the three QCL examples run at `f32`, `f64` and `Float106` (A, J).

**Connectors:** "as shown by" (the composition table), "more importantly" (the tolerance),
"similarly" (the coefficient field and the dual scalar).

**Rows answered:** P3, P6.

**Closing sentence direction:** A parameter that reaches the thresholds is a parameter worth
choosing deliberately, and choosing it deliberately has a price.

### Paragraph 9 — The pick, and what it costs (The economics)

**Main point.** Once precision is a parameter, the program author picks it per part against a stated
requirement, and the pick lands on a memory rung and a wall clock.

**Supporting points:**

- The trade-off, stated once: higher precision costs time and memory, lower precision saves both,
  and the right choice differs from one part of a simulation to the next. Library authors made that
  choice once for everyone, as well as they could, and a program inherited it with no way to revise
  it per part (A).
- Three computations bound three different ways, as the fourth code block: noise-bound (a Monte
  Carlo integral whose statistical `1/√n` buries rounding at any precision), mesh-bound (a central
  difference where truncation costs `h²/12` and rounding costs `ε/h²`), and reference-bound (the
  telescoping series, which earns a digit for roughly every three bits of mantissa) (A).
- The program measures each part at all three precisions against its closed form, states a budget
  per part, and picks the narrowest precision that meets it. The picks are `f32`, `f64` and
  `Float106`, and an assertion ties the written composition to the measured pick, so a changed
  budget fails loudly (A).
- Read the columns: the noise-bound column does not move with precision at all, the mesh-bound
  column moves once and stops, and the reference-bound column moves by ten digits and then by
  seventeen. The composed value lands at `5.9e-4`, which is the noise-bound part's own error, and
  the draws took 400 000 bytes where `Float106` would have taken 1 600 000 (A).
- **State the standing of the next table before showing it.** It is an estimate from stated
  assumptions on a convection-permitting ensemble forecast, not a measurement; every number follows
  from the assumption table by the arithmetic shown, so a reader with a different model can
  substitute their own (A).
- The parts, bound the same three ways: 40 ensemble members are noise-bound and go to `f32`, which
  is the finding that let ECMWF move its operational model to single precision (I13); the
  ensemble-variational assimilation is conditioning-limited and stays at `f64`; the conservation
  budgets are reference-bound, go to `Float106`, and cost a quarter of a megabyte because they are
  reductions (A).
- The result, as the closing table: state in memory 2182 GB against 1299 GB, a 40 % cut; the same
  384 cores one instance rung lower; list price `$22.62` against `$17.23` an hour; the run ending
  32 % sooner; total `$22.62` against `$11.72`, a 48 % cut, with the accuracy of the composed result
  set by the noise-bound part in both columns (A).
- **Separate the two halves of that 48 % explicitly.** The 24 % on the hourly rate is arithmetic on
  published list prices that the reader can redo. The 32 % on the clock rests on ECMWF's 40 %,
  measured on the IFS on ECMWF's machines and carried here as the best available figure rather than
  one measured on this model (A, I13).
- Bound the claim honestly: the saving is a step function of the instance ladder. It is largest
  where the `f64` state is pushed onto memory-priced instances, and it is zero at a node count where
  both states land on the same rung, where the halved byte traffic is what remains (A).

**Connectors:** "for instance" (the three bounds), "consequently" (the pick), "in particular"
(splitting the 48 %), "however" (the step function).

**Rows answered:** P3, P7.

**Closing sentence direction:** Same cores, same answer, half the memory, half the bill. The
difference is that somebody got to choose.

---

## The conclusion — razor sharp

**The deduction, stated in three sentences.** Eight pain points, five roots, one cause: a
translation written by hand where a type belonged. Removing the boundary removes the translation,
because there is no longer anything on either side of it to translate between. What the boundary
was hiding, in every case, was a decision: which convention, which primitive, which law, which
scalar. The decisions did not disappear when they were hidden; they were made by a library author,
once, before the reader's problem existed.

**The line to close on.** Precision was always a decision. It was made by whoever wrote the library
first, and the bill arrived as a memory rung and a wall clock that nobody itemised. It is a line in
your program now, and the table above is what it costs.

**What the post refuses to claim, stated immediately before that line, in three sentences:**

- Crossings at the outer edge of the stack are still translations. One is kept on purpose: a slice
  of `BFloat16` is byte-compatible with the `bf16` buffers accelerators exchange (A).
- The categorical layer is incomplete, and the `none` row of the trait table is the published work
  list (A).
- `Float106` costs two to four times an `f64` operation, and `∂ₖ ∘ ∂ₖ₊₁ = 0` is not checkable by the
  trait that requires it (A, J).

**Fallback close, if the primary reads too declarative for the venue:** "The mathematics was never
the hard part. The sentences between the libraries were, and there is no reason for a program to
contain any."

---

## Verb chain check before drafting

**Point 1 — build toward inevitability.** translates → drifts → degenerates → flips → survives →
corrupts. Hatton's *degenerated* is the peak; nothing after it in Point 1 should be stronger.

**Point 2 — build toward closure.** declares → projects → composes → owns → refuses → withholds →
proves. The turn is at *refuses*: the compiler refusing the blanket implementation, the trait
withdrawn where a law fails. That is the paragraph's argument in one verb.

**Point 3 — build toward consequence.** names → lifts → carries → derives → measures → picks →
costs. End on *costs*; it is the emphatic word of the whole post and it belongs at the end of a
sentence, not the middle.

**Weak verbs to avoid throughout:** is, has, can be, involves, provides, enables, allows, leverages,
supports.

**Verbs that must not repeat across paragraphs:** *reveals*, *demonstrates*, *ensures*. If one is
needed twice, the second use is a signal that two paragraphs are making the same point.

---

## Style gates (AiStyleguide, Elements of Style, the repo conventions)

Run these against the draft before it ships.

- **Em dashes:** at most one per 250 words, so at most 13 in a 3,200-word post. Count them. Where
  one appears, test a period, a semicolon and a comma first.
- **Semicolons:** present. Long-form prose with zero semicolons is the tell the styleguide names.
- **Sentence length:** target a 12-word range, not a 3-word range. Hatton's *six significant figures
  to one* deserves a short sentence on its own; the assumption table paragraph can run to thirty.
- **Paragraph openings:** no more than two of the nine may open with "Additionally", "Furthermore",
  "Moreover" or "In addition". Use the connectors listed per paragraph above; they are chosen for
  the relationship each actually carries.
- **Banned phrases:** delve into, shed light on, game-changer, unlock the potential, not only … but
  also, elevate, unleash, seamless, next-gen, robust, powerful.
- **Filler:** *very* and *really* below 2 % of words. Prefer deleting the intensifier to replacing it.
- **State, do not argue.** No "it is not A, it is B" constructions. No claim that no other stack
  does this. Describe what the code does and let the reader conclude (repo convention).
- **`FloatType` alias everywhere.** No raw `f64` in any snippet except at the display boundary, and
  no `as FloatType`, no `FloatType::from`, no locally defined conversion helper. Every crossing goes
  through `lift`, `lift_count`, `lower` or `to_count`, with a turbofish where an operator would
  leave the target type open (repo convention, A).
- **Every figure carries a source tag** from (A) through (J) in the draft. Strip the tags in the
  published version and keep a tagged copy beside it.

---

## Drafting notes

- Write Point 1 before reading the README again. The pain has to be the reader's before any answer
  lands, and familiarity with the solution is what flattens a problem statement.
- The three honesty moments are load-bearing and none of them may be softened: the entropy
  disagreement found at home (¶5), `∂∘∂ = 0` not checkable by the trait (¶6), and the economic
  section labelled an estimate before its table (¶9).
- Four code blocks, no more: the witness plus the rotor `fmap` (¶4), the telescoping series (¶7),
  the four-crate composition (¶8), the budget-and-pick program (¶9). Every one is running code in
  the tree; nothing is invented for the post.
- Four tables, no more: pain points trimmed to symptom and root (¶3), the four scalars (¶7), the
  composition residuals (¶8), the cost comparison (¶9). The witness table goes in ¶4 only if the
  post is running short; it is the first cut.
- Hatton carries Point 1 and ECMWF carries Point 3. Each is quoted once, in full, and not
  paraphrased elsewhere.
- The close drops rather than builds. Read the last three sentences aloud; if any of them needs
  emphasis to work, it is the wrong sentence.
