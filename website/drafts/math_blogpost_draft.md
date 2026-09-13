# Unified math, asked five ways: outline

Working title: **Every crossing is a translation**
Alternative: **Of unified math, the end is a number you can vouch for**

Reader: one person. A numerical scientist or Rust engineer whose simulation crosses at least two
math domains, a mesh and a tensor, a tensor and a rotor, a solver and a spectral step, and who has
spent an afternoon on the plumbing between them.

Thesis, one sentence: Every crossing between math domains is a translation done by hand, and
unified math removes the translations by defining container operations, scalar laws, and
conventions once, beneath every domain, so a result can be vouched for end to end.

Format: this outline follows `TurnRawMaterialintoWriting.pdf`. Part 1 lists the raw information
under headings, each item tagged with its source. Part 2 runs the four steps per section:
prioritize (topic sentence, paragraph count, closing sentence), connect (order and connectors),
transition, and the verb check. Part 3 maps the three Aristotelian rules onto the draft.

---

## Part 1: Raw information, by source

Sources:

- **(A)** `deep_causality_unified_math/README.md`
- **(B)** `website/docs/src/content/docs/concepts/uniform-math.md`
- **(C)** `examples/mathematics_examples/README.md`
- **(D)** `examples/mathematics_examples/composable_multi_math/README.md`
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

### Ethics: to what end?

- Every craft has an end that names it (E).
- Tensors and multivectors needed to compose; that need started the stack (A).
- "Nothing was converted between crates, because there was nothing to convert" (A).
- Residuals against closed forms: series `9.8e-31` at `Float106`, capstone drift `1.7e-31`, pipeline `Σ Δφ = 1.7e-32` (A, C, D).
- A precision chosen per part against a requirement, and the requirement has a price: `$22.62` against `$11.72` per run (A).
- A structure whose laws fail gets dropped rather than shipped: the multivector self-adjunction and the multivector `Monad` (B).
- "Nothing in this section was measured. Every number follows from the assumption table" (A).
- Default to `f64`; reach for more when the structure of the computation amplifies rounding (C).

### Epistemology: nature, origin, limits of knowledge

- Nature, three kinds: laws (trait bounds, compile errors, Lean theorems in `lean/THEOREM_MAP.md`), measurements (residuals), estimates (the weather ensemble) (A, B).
- Origin of the laws: standard abstract algebra and category theory; trait names follow them (B).
- Origin of the tower's shape: where laws fail. `3 - 5` has no value in ℕ; `5 · (1/5)` is `0` in ℤ; `ε` is a zero divisor in `Dual`; no rational `sqrt(2)` (B).
- Two witness defects were found by running code, not by reading source: `CausalTensorWitness` and `CausalMultiVectorWitness` violated monad right identity (A, G).
- `CsrMatrixWitness::bind` rebuilt every matrix as `1 × count`; right identity held only for a dense `1 × n` row (G).
- Measurement method: sum a series with a known closed form and read the rounding as digits: `BFloat16` 2, `f32` 4, `f64` 13, `Float106` 30 (A).
- `BFloat16` stops adding at `k = 23`; `f32` stalls past `k ≈ 4000`; `f64` rounds a million times and the roundings partly cancel (A).
- Limits: ε per scalar spans nine orders of magnitude; range takes two values (A, B).
- Limits: drift widens with precision only for chained transcendental work; rational arithmetic gains nothing from `Float106` (C).
- Limits: the picker needs a closed form or a reference; it answers `none` when no row meets the budget (A).
- Limits: the ECMWF `60 %` figure is transferred, not measured on this model (A).
- Limits: the witness table's `none` row: `Traversable`, `NaturalTransformation`, `Category`, `Kleisli`, `Bifunctor`, `Profunctor` (A).
- Limits: shaped containers stop at `Applicative`; a sparse `fmap` maps stored entries only (B).
- Lean proves the laws; it says nothing about rounding (B, A).

### Teleology: for what purpose does each thing exist or develop?

- The HKT interface wanted an algebra tower; the tower wanted numeric traits; composition then reached geometric algebra, exterior calculus, chain complexes, spectral methods, uncertainty (A).
- `fft` exists for one job: the spectral Poisson solve on periodic lattices, `388 ms` step to `1.9 ms` projection (B).
- `homology` carries no geometry, so a quantum error-correcting code uses it without a mesh (B).
- `stats` exists so `tensor` and `uncertain` hand their reductions over, which puts `stats` on the longest chain (A, B).
- `metric` exists so general relativity in `tensor` and particle physics in `multivector` share sign conventions (B).
- `lift` exists because `x as FloatType` and `FloatType::from(0.5)` each fail on one of the four scalars (A).
- `BFloat16` exists for byte-compatible accelerator buffers; `Float106` for reference values and oracles (A, C).
- A number system gets a crate only when it introduces a type Rust lacks: ℂ and ℚ do, ℕ, ℤ, ℝ do not (B).
- The purpose of a part decides its precision: noise-bound `f32`, mesh-bound `f64`, reference-bound `Float106` (A).
- The witness table is the work list, ranked from mechanical to design fork (A, G).
- Consumers outside the folder: `quantum`, `physics`, `cfd`, `algorithms`, `discovery` (B).
- The same composition law composes a `PropagatingEffect` from the effect propagation process (B).

### Metaphysics: the fundamental nature of what exists

- "The tower states laws, not representations" (B).
- One answer to "what laws does this number obey", never one per crate (B).
- Set name against algebraic name: `Integer` and `EuclideanDomain` name the same type for two questions (B).
- Markers are parameterised by operator, so `(𝔽₂, ×)` commuting says nothing about `(𝔽₂ᵐˣⁿ, ×)` (B).
- A witness is a zero-sized struct standing in for a type constructor; it claims a trait only when the laws hold (B).
- The removed adjunction returned `3` where the law required `4`, and no `unit` could do better (B).
- `FloatType` selects precision; `IntType` selects range. Rounding is a graded error; overflow is a hard wrongness (B).
- Precision is a parameter of the program, not a property of it: one program, four scalars (A).
- Stokes' theorem is an `Adjunction` type: `⟨dω, C⟩ = ⟨ω, ∂C⟩` (B).
- The structure of the code matches the structure of the math (B).
- The dependency graph is drawn as its transitive reduction; no crate depends on a crate above it (A, B).

### Ontology: the types of things and how they relate

- Seventeen crates, eight tiers, `num` and `metric` at tier 0, `topology` at tier 7 (A).
- The tower branches: `Semiring` is `Ring` with additive inverses removed; nothing on the ℕ branch climbs across (B).
- Number sets: ℕ, ℤ, ℚ, ℝ, ℂ, plus `Gf2`, `Dual`, `Quaternion`, `Octonion` (B).
- Four real fields: `BFloat16`, `f32`, `f64`, `Float106` (A).
- Containers: `CsrMatrix`, `DenseMatrix`, `DenseVector`, `CausalTensor`, `CausalTensorTrain`, `CausalMultiVector`, `CausalMultiField`, `Manifold`, any `ChainComplex`, `PropagatingEffect` (B).
- Witness traits implemented: `Functor`, `Foldable`, `Pure`, `Applicative`, `Monad`, `CoMonad`, `Semigroupal`, `MonoidalApplicative`, `Adjunction`, `Arrow` (A, B).
- Relations: depends-on (tiers), bounds-on (weakest law that works), implements (witness table), nests (`CausalTensor<CausalMultiVector<T>>`), reaches (closure inside `extend`), crosses (`lift`, `lift_count`, `lower`, `to_count`), consumed-by (A, B).
- Bounds table: transpose on `CommutativeSemiring`; `determinant_exact` on `EuclideanDomain`; `rref` on `Field`; SVD on `ConjugateScalar` (B).
- `metric` sits horizontally with no dependencies (B).
- No required external dependency; four optional ones behind feature gates (A).
- `CausalFlow` hands the state to each step and short-circuits on the first error (B).

### Ethos, pathos, logos material

- Ethos: expertise shown, integrity shown, veritas (F). Measured tables with the program that printed them (A). Defects recorded with the probe (G). Removals stated (B). The estimate labelled as estimate (A). The pain-point assessment labelled as the author's (H).
- Pathos: undivided attention to the reader; emotions are contagious; reach out and invite (F). The reader's own afternoon on plumbing (P1), the sign that passed the tests (P2), the drift with no owner (P3). The reader who signs for the nodes (P7).
- Logos: premise, deduction, conclusion (F). See Part 3.

---

## Part 2: The steps, per section

Order: the pain (assessment), the end (Ethics), what exists once (Ontology), what it is made of
(Metaphysics), how it is known and where knowing stops (Epistemology), why each piece came to be
(Teleology), the pain list revisited with the price (close). The assessment opens because the
reader must recognise the problem before any answer means anything; the inventory precedes the
claims about it; the limits precede the purposes so the purposes never read as promises.

Logos spine: each section closes on the premise of the next. Estimated length: 21 paragraphs,
about 3,400 words, plus four code blocks and six tables.

### 0. The assessment: eight translations

**Main point.** Every pain point in a cross-domain simulation is a translation done by hand at a
boundary, and translations are where errors lose their owner and costs lose their line item.

**Topic sentence.** A simulation that crosses two math domains pays at every boundary, and the
payments have a shape: each one is a translation the programmer does by hand.

**Paragraphs.** 3: the reader's scene; the eight translations as a table with one line each; what
the translations have in common.

**Middle, in order.**
1. (To illustrate) The scene: a mesh walk in one library, a contraction in a second, a rotor in a third; the afternoon goes to the plumbing (P1) (B, H).
2. (Then) The sign that passed the tests (P2); the drift with no owner (P3); the elimination fed integers (P4); the two variances (P5); the rotor field flattened to a buffer (P6); the `f64` bill on a noise-bound part (P7); the `NaN` that reached the average (P8). Show the pain-point table, trimmed to symptom and root (H).
3. (In other words) Each row translates one of five things by hand: a container, a convention, a scalar, a law, or a primitive.
4. (Consequently) A translation done by hand has no owner when it fails and no line item when it costs, which is why the drift and the bill both arrive unexplained.
5. (Qualification) State the assessment as the author's own, drawn from practice and from the published record; give each row one citation from I1 to I13 in a footnote; invite the reader to strike rows that do not match theirs.

**Closing sentence.** The eight rows reduce to one question: what would a stack look like in which
the five translated things were defined once, beneath every domain, so that the boundaries carried
nothing to translate.

**Transition.** Aristotle would answer by naming the end first.

**Code to carry.** The pain-point table (H).

**Verbs.** pays, translates, goes, passes, flattens, reaches, loses, arrives, reduces, carries.

### 1. Ethics: to what end?

**Main point.** The end of unified math is a result vouched for across every crossing.

**Topic sentence.** Of medicine the end is health, and of unified math the end is a result its
author can vouch for from the first boundary to the last.

**Paragraphs.** 2.

**Middle, in order.**
1. (Because) Vouching across a crossing takes three things: composition with nothing converted (P1, P6), a residual measured against a closed form at the composed level (P3), and a cost stated per part (P7) (A).
2. (For instance) Composition: a tensor of multivectors rotated by one `fmap`; show the two-line snippet (A).
3. (For instance) Residual: the capstone lands `1.7e-31` from `(cosh θ, sinh θ)` at `Float106`, measured after four crates have touched the value (D).
4. (For instance) Cost: `$22.62` against `$11.72` per run, from a stated assumption table (A).
5. (In particular) The end also fixes what the craft refuses: a structure whose laws fail is dropped (P4), and an estimate is labelled an estimate (B, A).

**Closing sentence.** Health is the end of medicine whether or not one patient recovers; the vouched
result is the end of unified math whether or not one program reaches it, and reaching it depends on
what the stack defines once.

**Transition.** So the inventory comes next, read as the list of things defined once.

**Code to carry.** The `fmap` rotation, two lines (A).

**Verbs.** vouch, takes, rotates, lands, costs, refuses, drops, labels, defines.

### 2. Ontology: what exists once, and how it relates

**Main point.** Seventeen crates in eight tiers define the container operations, the scalar laws,
the conventions, and the primitives once, and every crate depends only on crates below it.

**Topic sentence.** Seventeen crates in eight tiers hold everything unified math contains, and each
of the five translated things lives in exactly one place in that graph.

**Paragraphs.** 3: the tiers and where each translated thing lives; the numbers and the tower; the
containers and the relations.

**Middle, in order.**
1. (First) `num` holds the scalars, `metric` the conventions, `algebra` the laws, `haft` the container operations, `stats` and `rand` the primitives; show the tier ladder and mark the five (A, B). P2 and P5 answered here.
2. (Second) The tower branches at `Semiring` against `Ring`; ℕ stays on its branch because `3 - 5` has no value there (B).
3. (Then) Five sets plus four more types; a set gets a crate only when it adds a type Rust lacks (B).
4. (Then) Ten containers, each with a witness naming the operations it admits; a container of one crate holds values of another (P1, P6) (B, A).
5. (Finally) Seven relations: depends-on, bounds-on, implements, nests, reaches, crosses, consumed-by. Show the bounds table (B).
6. (Also) `metric` sits horizontally; `CausalFlow` short-circuits on the first error (P8); five consumers stand outside the folder (B).

**Closing sentence.** The inventory is one ordered graph, scalars at the bottom, laws over them,
containers over the laws, and each of the five translated things appears in it once.

**Transition.** A witness admits an operation only when its laws hold, and that condition says what
these things are made of.

**Code to carry.** The tier ladder with the five translated things marked (A); the weakest-law
bounds table (B).

**Verbs.** hold, live, depend, branch, stay, admit, nest, sit, short-circuit, appear.

### 3. Metaphysics: what these things are made of

**Main point.** A thing in unified math is what it obeys, and it exists on the surface only while
its laws hold; this is what turns P4 from a wrong number into a compile error.

**Topic sentence.** In unified math a number is what it obeys: the tower states laws, and the
representation follows.

**Paragraphs.** 3: laws define the scalar; laws admit or remove a container; two natures of number.

**Middle, in order.**
1. (For instance) `i64` reaches `EuclideanDomain` and stops short of `Field` because `5 · (1/5)` is `0`; `Dual` reaches `Real` and stops short of `Field` because `ε` divides zero; `Rational` reaches `Field` and stops short of `Real` because no rational squares to `2`. Show the stopping table (B). The elimination fed integers (P4) fails to compile.
2. (In other words) One type answers two questions: `Integer` names the set, `EuclideanDomain` names the structure (B).
3. (Consequently) A container exists on the composition surface by claiming a trait, and it loses the claim when a law fails: the multivector self-adjunction returned `3` where the law required `4`, so it went; the multivector `Monad` went with it; the tensor `bind` was fixed to keep shape (B, A).
4. (Similarly) A convention is a type: `Metric::Minkowski(4)` against `Metric::Lorentzian(4)` names the sign convention, and Stokes' theorem exists as an `Adjunction` type (P2) (B).
5. (In contrast) Two natures of number: ℝ is approximated with a graded error bounded by ε; ℤ is exact in a window and fails hard by overflow. `FloatType` selects precision, `IntType` selects range (B).
6. (Building on that) One program at four scalars is the same program; precision is a parameter of the program, not a property of the library (P3, P7) (A).

**Closing sentence.** What exists here exists by law, and two courts decide it: the compiler on every
build and the Lean checker on every theorem.

**Transition.** Two courts raise the question how anything is known here, and how far the knowing
reaches.

**Code to carry.** The stopping table (B); the `Adjunction` line for Stokes (B).

**Verbs.** obeys, states, reaches, stops, divides, claims, loses, went, fixed, names, approximates,
fails, selects, decides.

### 4. Epistemology: what is known, how, and where it stops

**Main point.** Unified math knows three kinds of thing, proven laws, measured residuals, and stated
estimates, and it labels which is which; this is what gives the drift of P3 an owner.

**Topic sentence.** Unified math knows three kinds of thing, and it labels each: laws it proves,
residuals it measures, and estimates it states.

**Paragraphs.** 3: the three kinds; the measurement method; the limits.

**Middle, in order.**
1. (First) Laws: a wrong bound is a compile error; the laws themselves bind to Lean theorems (B).
2. (Second) Measurements: sum a series with a closed form and read the rounding as digits. Show the program and the four-row table (A).
3. (For example) `BFloat16` stops adding at `k = 23` because the term falls below half a unit in the last place; `f64` rounds a million times and the roundings partly cancel (A).
4. (Then) The drift gets an owner: run the composed pipeline at `f32`, `f64`, and `Float106` and watch the residual move twenty-five orders of magnitude together; a step that fails to move is the step that owns the drift (P3). Show the pipeline table (A).
5. (Third) Estimates: the weather ensemble follows from an assumption table by shown arithmetic; the ECMWF `60 %` is transferred (A).
6. (Significantly) Two defects were measured, not read: `bind(m, pure)` returned `(1, 4)` for a `(2, 2)` input (G).
7. (However) Limits: ε floors each scalar; drift widens only for chained transcendental work; the picker needs a reference and answers `none` without one; the witness table's `none` row marks the edge of the surface; a sparse `fmap` touches stored entries only; Lean proves laws and says nothing about rounding (A, B, C).

**Closing sentence.** Knowledge here carries a provenance column, proven, measured, or assumed, and
the column stays visible in every table the post shows.

**Transition.** Each thing in that inventory came to exist for a reason, and the reasons explain the
shape of the stack.

**Code to carry.** The telescoping program and its digits table (A); the pipeline residual table (A).

**Verbs.** knows, labels, proves, measures, states, stops, falls, cancels, moves, owns, transfers,
floors, widens, marks.

### 5. Teleology: why each piece exists, and where the work points

**Main point.** Each tier exists for the tier that asked for it, each single-purpose crate answers
one translation, and development follows the gap list.

**Topic sentence.** The stack grew downward from one need, tensors and multivectors had to compose,
and each tier exists for the tier that asked for it.

**Paragraphs.** 3: the descent; the single-purpose crates as answers to P2, P5, P7; precision per
purpose and the work list.

**Middle, in order.**
1. (Initially) The shared interface wanted an algebra tower; the tower wanted numeric traits; composition then reached exterior calculus, chain complexes, spectra, uncertainty (A).
2. (For instance) `metric` exists so two physics share one sign convention (P2); `stats` exists so two crates stop carrying their own variance (P5); `lift` exists because two obvious spellings each fail on one scalar (P3); `fft` exists for one solve, `388 ms` to `1.9 ms`; `homology` carries no geometry so a code with no cells can use it (A, B).
3. (Likewise) `BFloat16` exists for accelerator buffers, the one bridge kept on purpose; `Float106` exists as the oracle (A, C).
4. (Accordingly) Purpose decides precision, and the program author decides it per part (P7): noise-bound `f32`, mesh-bound `f64`, reference-bound `Float106`. Show the pick table (A).
5. (Looking forward) The witness table's `none` row is the work list, ranked mechanical to design fork (A, G).
6. (Above the folder) Five consumers, and the causal monad the whole stack serves (B).

**Closing sentence.** Purpose runs in both directions, down through the tiers to the scalar and up to
the causal models that consume the stack.

**Transition.** Which returns the post to the eight rows it opened with.

**Code to carry.** The mixed-precision pick program and table (A).

**Verbs.** grew, asked, wanted, reached, exists, share, stop, fail, decides, ranks, serves.

### 6. Close: the eight rows, revisited, and the price

**Main point.** Seven of the eight translations are gone, one is kept on purpose, and the weather
ensemble puts a price on the difference.

**Topic sentence.** Read the eight rows again with the stack in hand: seven translations have
nothing left to translate, and the eighth is kept where the stack meets an accelerator.

**Paragraphs.** 3: the rows revisited; the price; the rule and the invitation.

**Middle, in order.**
1. (First) One line per row: P1 and P6 by nesting and closure reach; P2 by `metric`; P3 by the alias and the residual tables; P4 by bounds and removals; P5 by `stats` and `rand`; P7 by the pick; P8 by `CausalFlow`. Show a two-column table, row and answer (A, B).
2. (However) What stays: the outer edge, the `none` row, the cost of `Float106`. State each once (A).
3. (Because) The rule is precision per part against a requirement; here the requirement is a memory rung and a wall clock (A).
4. (Specifically) `$22.62` against `$11.72`, `48 %`, with the two effects kept apart: `24 %` is list-price arithmetic, `32 %` is a transferred measurement (A).
5. (For the reader) Default to `f64`; reach for more when the structure amplifies rounding; measure against a closed form; say what was measured and what was assumed (C, A).
6. (Invitation) The three commands to run (A).

**Closing sentence.** Of medicine, health; of shipbuilding, a ship; of unified math, a number you
can vouch for across every crossing, at a cost you chose.

**Code to carry.** The rows-revisited table (H, A, B); the master comparison table trimmed to six
rows (A); the three `cargo run` lines (A).

**Verbs.** read, translate, keep, stay, price, keep apart, default, reach, measure, say, run.

---

## Part 3: The three Aristotelian rules, applied

**Logos: premise, deduction, conclusion.**
- Premise 1: every pain point in a cross-domain simulation is one of five things translated by hand at a boundary: a container, a convention, a scalar, a law, or a primitive.
- Premise 2: a translation done by hand has no owner when it fails and no line item when it costs.
- Evidence for premise 1: faults cluster at interfaces, one in seven in Fortran (I1); the sign flip that retracted five papers crossed a lab boundary in a supplied program (I2, I3); the Mars Climate Orbiter's units crossed a software interface (I10); array libraries differ just enough that code cannot cross them (I8).
- Evidence for premise 2: agreement fell from six significant figures to one with no single owner across independent implementations (I1); the unprotected conversion on Ariane 5 had a workload budget and no owner for its range (I11); the single-precision saving at ECMWF was a line item only once someone measured it (I13).
- Deduction: define each of the five once, beneath every domain, and the boundaries carry nothing to translate; a result can then be measured and priced end to end.
- Conclusion: "Nothing was converted between crates, because there was nothing to convert"; the digits table, the pipeline table, the pick, the price.
- Each section ends on the premise of the next; the close returns to the eight rows.

**Ethos: credibility carried by evidence.**
- The assessment is labelled as the author's, drawn from practice, and every row carries a published citation from I1 to I13; the reader is invited to strike rows.
- Quotations are reproduced verbatim from the sources listed; a figure the post cannot trace to a listed line is cut.
- Every number cites the program that printed it and the scalar it ran at.
- Each removal gets one sentence with the failing input: `3` where `4` was required.
- The estimate section keeps its first sentence: nothing in it was measured.
- The Lean binding gets one sentence and a link to `lean/THEOREM_MAP.md`.
- Any wall-clock figure names the machine it ran on.

**Pathos: undivided attention to one reader.**
- Open on the reader's own afternoon: the plumbing, the sign that passed, the drift with no owner.
- Keep the tone of someone who found a defect and fixed it, never of someone selling.
- Close with an invitation the reader can act on in one minute: three commands.
- The price table speaks to the reader who signs for the nodes.

---

## Part 4: The verb check, before each paragraph is written

Weak verbs to strike on sight: is, are, has, provides, allows, enables, supports, leverages,
offers, features, utilizes, handles.

Strong verbs the material supplies: pay, translate, flatten, vouch, compose, rotate, land, cost,
drop, refuse, hold, depend, branch, admit, nest, obey, state, stop, claim, lose, fix, name,
approximate, overflow, decide, prove, measure, label, stall, cancel, move, own, transfer, floor,
widen, grow, ask, want, reach, serve, pick, price, run.

Per paragraph, the four questions from the guide: strong or weak; does the chain tell the story;
do the verbs point back at the main point; any repeats.

## Part 5: House rules for the draft

- No em-dashes; no "not A, it's B"; no uniqueness claims; state, do not argue.
- The pain-point assessment names symptoms and roots; it names no other library.
- Real code or real output in every section, cut from the README programs.
- `FloatType` alias in every snippet; `f64` only at the display boundary.
- Say "project website" where the website comes up.
- Sentence length varies; the connector matches the relation it signals.
- Six revision passes from `ElementsOfStyle.md`: cut, verb, order, cohesion, voice, reader.
