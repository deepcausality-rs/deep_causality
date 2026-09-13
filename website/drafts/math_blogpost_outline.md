# Every crossing is a translation: outline

Working title: **Every crossing is a translation**
Alternative: **Glue once, then map**

Reader: one person. A numerical scientist or Rust engineer whose simulation crosses at least two
math domains, a mesh and a tensor, a tensor and a rotor, a solver and a spectral step, and who has
spent an afternoon on the plumbing between them.

Thesis, one sentence: Every crossing between math domains is a translation done by hand at the call
site, and unified math moves each translation into one tested place, gives every container the same
operations, and makes precision a parameter, so a program composes across domains without glue and
reruns at any precision from one edit.

Three points carry the argument, and the sections follow them:

1. **The witness inverts the glue.** Conventional code converts structure at every call site. Unified math converts it once, in the container's HKT witness, tests the laws there, and the call site says `fmap`, `bind`, or `extend`.
2. **One pattern, every container.** Every crate in the stack implements the same traits with the same conventions, so learning the pattern once covers ten containers and every crossing between them.
3. **Precision is a parameter, and it costs nothing to switch.** One alias, four scalars, no second copy of the math, and mixed precision per part against a stated budget.

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

### The three points: raw material

Additional sources:

- **(J)** Code on `main`: `deep_causality_unified_math/deep_causality_tensor/src/extensions/ext_hkt.rs`, its tests in `tests/extensions/causal_tensor_ext_hkt_tests.rs`, `deep_causality_topology/src/extensions/hkt_manifold/mod.rs`, and `examples/mathematics_examples/composable_multi_math/triple_hkt_stress_field/main.rs`

#### Point 1: the witness inverts the glue

- Rust has no native higher-kinded types; HAFT adds them with a witness, a zero-sized struct that stands in for the type constructor (B).
- A crate that owns a container generic in its element declares a witness, binds `type Type<T>` to the container, and implements the categorical traits against the witness (A).
- The tensor witness, in full: `impl HKT for CausalTensorWitness { type Type<T> = CausalTensor<T>; }` and an `fmap` that reads the shape, maps the flat vector, and rebuilds (J). Eight lines.
- The laws are tested at the witness, once: `test_monad_causal_tensor_right_identity`, `_left_identity`, `_associativity`, and the zip witness's semigroupal associativity (J).
- Two defects were found by those law tests, not by reading: `CsrMatrixWitness::bind` rebuilt every matrix as `1 × count`; `CausalTensorWitness` violated right identity and was fixed to keep the input's shape (G, A).
- Where a law cannot hold, the trait is withdrawn rather than shipped: `CausalMultiVectorWitness` gave up `Monad`; the shaped `linear` witnesses stop at `Applicative` (B, A).
- The witness docstring records the corner it cannot close: a one-element tensor can carry `[]`, `[1]` or `[1, 1]`, and `bind` must choose; right identity wins, associativity parts company on that input, and the note says so (J).
- Nesting: a witness accepts any element type, including one another crate owns. `CausalTensor<CausalMultiVector<FloatType>>` is an ordinary tensor, and one `fmap` rotates every cell by a rotor (A).
- Closure reach: `extend` hands a cursor to a closure, and the closure may call any crate (A).
- The call site after inversion, two lines: `CausalTensorWitness::fmap(field, |v| rotor.geometric_product(&v).geometric_product(&rotor_rev))` (A).
- Arithmetic of the inversion, the author's (H): with `K` container types, pairwise bridges number up to `K(K − 1)`, one per ordered pair, each written and tested where it is used. Witnesses number `K`, one per container, tested once. At `K = 10` that is ninety possible bridges against ten witnesses.
- Hatton's rate applies to the bridges: one fault per seven Fortran interfaces (I1). Fewer interfaces, fewer places for that rate to act.

#### Point 2: one pattern, every container

- `fmap`, `bind`, `extend`, and `extract` mean the same thing on a tensor, a matrix, a multivector, a manifold, and a propagating effect (B).
- Ten containers, one witness each, one trait vocabulary. The table is the evidence for the point and goes into the draft before the claim that the pattern transfers; the rightmost column says which operations a container admits and, by omission, which it refuses (B):

| Crate | Container | Witness | Implements |
|---|---|---|---|
| `linear` | `CsrMatrix<T>` | `CsrMatrixWitness` | Functor, Foldable, Pure, Applicative, CoMonad |
| `linear` | `DenseMatrix<T>` | `DenseMatrixWitness` | Functor, Foldable, Pure, Applicative, CoMonad |
| `linear` | `DenseVector<T>` | `DenseVectorWitness` | Functor, Foldable, Pure, Applicative, Monad, CoMonad |
| `tensor` | `CausalTensor<T>` | `CausalTensorWitness` | Functor, Foldable, Pure, Applicative, Monad, CoMonad |
| `tensor` | `CausalTensor<T>`, zipped | `ZipTensorWitness` | Functor, Semigroupal, MonoidalApplicative, Convolutional |
| `tensor` | `CausalTensorTrain<T>` | `CausalTensorTrainWitness` | Functor, Foldable, Pure |
| `multivector` | `CausalMultiVector<T>` | `CausalMultiVectorWitness` | Functor, Foldable, Pure, Applicative, CoMonad |
| `multivector` | `CausalMultiField<T>` | `CausalMultiFieldWitness<T>` | Functor, Pure, CoMonad |
| `topology` | `Manifold<C, F>` | `ManifoldWitness<C>` | Functor, Foldable, Pure, Applicative, Monad, CoMonad |
| `topology` | any `ChainComplex` | `GenericManifoldWitness<K>` | Functor |
| `core` | `PropagatingEffect<T>` | `PropagatingEffectWitness<E, L>` | the causal-monad stack |

- Read the table two ways. Down a column, the same trait name means the same operation on every row, which is the transfer claim. Across a row, a missing name is a law that does not hold: the shaped `linear` witnesses and `CausalMultiVectorWitness` stop short of `Monad` because right identity fails on them (B, G).
- `deep_causality_topology` ships twelve witnesses; graphs, hypergraphs, mixed graphs, cell complexes, lattice complexes, point clouds, chains, boundaries, exterior derivatives all answer to the same `extend` (B, J).
- The generic function that proves it: `double_value::<OptionWitness>`, `::<VecWitness>`, `::<CausalTensorWitness>` all type-check on one body (B).
- The stress example: six steps, strain to von Mises, inside one `ManifoldWitness::extend`; topology supplies the walk, tensor the constitutive law and contraction, multivector the material rotor in `Cl(3,0)`; output `2.403e8 Pa` at the two vertices with the largest `x`, zero at the three with `x = 0` (D, J).
- Each step in that closure is a standalone function, testable alone, replaceable alone (D).
- The GRMHD chain: tensor, metric selection, multivector, tensor, scalar branch; five regimes, one `CausalFlow`, short-circuit on the first error (B).
- The conventions are shared the same way: one `metric` crate, so `Cl(3,1)` in `multivector` and the Einstein tensor in `tensor` name the same signature; one `stats` crate, so `tensor` and `uncertain` hand their reductions over (A, B).
- The gaps are listed, not hidden: the `none` row of the trait table is the work list (A).
- The capstone: four crates, one alias, drift `1.7e-31` against `(cosh θ, sinh θ)` at `Float106` (D).
- Contrast, the author's (H): conventional code learns one iteration idiom per library, one broadcasting rule per library, one error convention per library; the unified stack learns one witness pattern and reads the trait table for what each container admits.

#### Point 3: precision as a parameter, for free

- Every crate above `num` is generic in its scalar; the bounds `Real`, `RealField`, `Scalar` are what a tensor, manifold or multivector asks of its element, and the four shipped real fields satisfy them (A).
- One line: `type FloatType = f64;` Switch it and the arithmetic of the whole program changes precision; nothing else moves (A, C).
- No parallel implementation, no second copy of the math (B).
- The four scalars: `BFloat16` 2 bytes and 2 digits; `f32` 4 bytes and 7; `f64` 8 bytes and 16; `Float106` 16 bytes and 31 (A).
- `Float106` costs two to four times an `f64` operation (A, as corrected on `main`). `BFloat16` halves memory and matches accelerator `bf16` buffers byte for byte (A).
- The three crossings a program cannot avoid, written once in `lift`: literal in, count in, display out; the two obvious spellings each fail on one scalar (A).
- Measured: the telescoping series, a million terms, `BFloat16` 2 correct digits, `f32` 4, `f64` 13, `Float106` 30 (A).
- Measured across four crates: `Σ Δφ` at `2.0e-7`, `6.9e-17`, `1.7e-32`; a thousand geometric products at `1.5e-7`, `1.6e-13`, `1.3e-29`; "Nothing was converted between crates, because there was nothing to convert" (A).
- When it pays: drift widens with precision only for chained transcendental work; rational arithmetic gains nothing from `Float106` (C).
- Mixed precision as a parameter: noise-bound `f32`, mesh-bound `f64`, reference-bound `Float106`; the program measures each part against its closed form, states a budget, and picks the narrowest precision that meets it; an assertion ties the code to the pick (A).
- The composed value lands at `5.9e-4`, the noise-bound part's own error, and the draws took `400 000` bytes at `f32` against `1 600 000` at `Float106` (A).
- Priced: the forecast ensemble, `2182 GB` against `1299 GB` of state, one instance rung lower, `$22.62` against `$11.72` per run, `48 %`, of which `24 %` is list-price arithmetic and `32 %` is ECMWF's measurement transferred (A, I13).
- Contrast, the author's (H): conventional stacks fix the scalar per library, so a mixed pipeline is a chain of silent downcasts and a wider rerun means a second implementation.

---

## Part 2: The steps, per section

Order: the assessment; the witness inverts the glue; one pattern, every container; precision as a
parameter; the eight rows revisited with the price. Each of the three middle sections opens on the
conventional way, shows the unified way in real code, and names the rows it answers. The assessment
opens because the reader must recognise the problem before any answer means anything.

Logos spine: each section closes on the premise of the next. Estimated length: 19 paragraphs,
about 3,200 words, four code blocks, five tables.

### 0. The assessment: eight translations

**Main point.** Every pain point in a cross-domain simulation is a translation done by hand at a
boundary, and translations are where errors lose their owner and costs lose their line item.

**Topic sentence.** A simulation that crosses two math domains pays at every boundary, and the
payments share a shape: each is a translation the programmer writes by hand.

**Paragraphs.** 3: the reader's scene; the eight translations, one line each; what they have in
common.

**Middle, in order.**
1. (To illustrate) The scene: a mesh walk in one library, a contraction in a second, a rotor in a third; the afternoon goes to the plumbing (P1) (B, H).
2. (Then) The sign that passed the tests (P2, I2); the drift with no owner (P3, I1); the conversion nobody guarded (P4, I11); the two variances (P5, I12); the rotor field flattened to a buffer (P6); the `f64` bill on a noise-bound part (P7, I13); the `NaN` that reached the average (P8, I6). Show the pain-point table trimmed to symptom and root.
3. (In other words) Each row translates one of five things by hand: a container, a convention, a scalar, a law, or a primitive.
4. (Consequently) A hand translation has no owner when it fails and no line item when it costs; Hatton measured the first, six significant figures falling to one (I1), and ECMWF measured the second, forty percent (I13).
5. (Qualification) State the assessment as the author's, drawn from practice and the published record, one citation per row, and invite the reader to strike rows that do not match theirs.

**Closing sentence.** The eight rows reduce to one question: where should a translation live, if
the call site is the wrong place.

**Transition.** The first answer moves it one level down, into the container itself.

**Code to carry.** The pain-point table (H, I).

**Verbs.** pays, writes, goes, passes, flattens, reaches, loses, measured, reduces, lives.

### 1. The witness inverts the glue

**Main point.** Unified math writes each container's translation once, in its witness, tests the
laws there, and leaves the call site with a single verb.

**Topic sentence.** Conventional code converts structure where it is used; unified math converts
it where it is defined, once, and tests it there.

**Paragraphs.** 4: the conventional shape; the witness; the law tests and what they caught; the
call site after inversion.

**Middle, in order.**
1. (First) The conventional shape: `K` container types need up to `K(K − 1)` pairwise bridges, each written at a call site and tested, if at all, there; at ten containers that is ninety possible bridges (H). Hatton's one fault per seven interfaces acts on every one of them (I1).
2. (Instead) The witness: a zero-sized struct binds `type Type<T>` to the container and implements the traits against itself (B, A). Show the tensor witness, the `HKT` and `Functor` blocks, eight lines (J).
3. (Because) The laws are tested at the witness, once: right identity, left identity, associativity, named tests (J). Those tests found the sparse `bind` that rebuilt every matrix as one row and the tensor `bind` that returned `[6]` for `[2, 3]` (G, A). One was fixed; where a law could not hold, the trait was withdrawn (B).
4. (Even so) The witness docstring records the corner it cannot close, the one-element tensor whose shape `bind` must choose, and says which law wins and which parts company (J). That note is the honest edge of the inversion.
5. (Then) The call site: two lines, one `fmap`, a tensor of multivectors rotated by one rotor (A). Nesting puts another crate's value inside this crate's container; closure reach lets `extend` call any crate from inside the walk (A).
6. (So) Rows P1, P4 and P6 close here: no repacking at the boundary, laws checked where the container lives, and the rotor field never flattens.

**Closing sentence.** Ten witnesses replace ninety bridges, and each witness carries its law tests
with it, so the translation has an owner and a test before any call site exists.

**Transition.** Ten witnesses are only a saving if they are ten of the same thing.

**Code to carry.** The `HKT` and `Functor` impl for `CausalTensorWitness` (J); the three law-test
names (J); the two-line `fmap` rotation (A).

**Verbs.** converts, binds, implements, tests, found, fixed, withdrew, records, rotates, nests,
reaches, replaces, carries.

### 2. One pattern, every container

**Main point.** Every crate in the stack implements the same traits with the same conventions, so
the pattern learned on one container works on all ten and on every crossing between them.

**Topic sentence.** The same four verbs, `fmap`, `bind`, `extend`, `extract`, mean the same thing
on a tensor, a matrix, a multivector, a manifold, and a propagating effect.

**Paragraphs.** 4: the conventional cost of learning; the trait table; the six-step closure; the
shared conventions underneath.

**Middle, in order.**
1. (First) Conventional stacks charge per library: one iteration idiom, one broadcasting rule, one error convention, one sign convention each, and the array API consortium's finding that the APIs differ just enough to stop code from crossing (I8, H).
2. (Instead) One trait vocabulary and one table that says which container admits which operation; show the witness table from Part 1 here, in full, before any claim that the pattern transfers, and read it down a column and across a row (B). A generic function over the witness runs on `Option`, `Vec` and `CausalTensor` from one body (B).
3. (For instance) Six steps from strain to von Mises inside one `ManifoldWitness::extend`: topology walks, tensor contracts, multivector rotates in `Cl(3,0)`; `2.403e8 Pa` where the prescribed strain is largest, zero where it vanishes (D, J). Show the closure, ten lines. Each step is a standalone function, replaceable alone (D).
4. (Likewise) The GRMHD chain crosses five regimes in one `CausalFlow`, and a failure in step two never reaches step three (B). Row P8 closes here.
5. (Underneath) The conventions are shared the same way the traits are: one `metric` crate names `Cl(3,1)` for both tensor and multivector code, one `stats` crate serves both `tensor` and `uncertain` (A, B). Rows P2 and P5 close here.
6. (Honestly) The trait table has a `none` row, and it is the work list, ranked from mechanical to design fork (A, G).

**Closing sentence.** One pattern, learned once, reaches ten containers, twelve topology witnesses,
and every crossing between them, because the crates agreed on the verbs before they agreed on
anything else.

**Transition.** The crates agreed on one more thing: none of them names its scalar.

**Code to carry.** The witness table (B); the six-step `extend` closure (J); the GRMHD chain,
trimmed to three steps (B).

**Verbs.** mean, charge, admits, runs, walks, contracts, rotates, replaces, crosses, reaches, names,
serves, agreed.

### 3. Precision as a parameter, for free

**Main point.** One alias sets the precision of the whole program, switching it costs one edit and
no second implementation, and the same mechanism buys mixed precision per part.

**Topic sentence.** Every crate above `num` is generic in its scalar, so a program names its working
type once and writes everything else against the name.

**Paragraphs.** 5: the conventional lock-in; the alias and the lifts; the digits table; the
cross-crate residuals; mixed precision and its price.

**Middle, in order.**
1. (First) Conventional stacks fix the scalar per library, so a mixed pipeline is a chain of silent downcasts, and a wider rerun to find a drift means a second implementation (H). Hatton's six figures falling to one had no oracle to fall against (I1).
2. (Instead) `type FloatType = f64;` and four scalars that satisfy the same bounds; show the alias and the scalar table (A). The three unavoidable crossings, literal in, count in, display out, are written once in `lift`, because the two obvious spellings each fail on one scalar (A).
3. (Measured) A million terms of a telescoping series: 2, 4, 13, 30 correct digits, one line changed between rows; `BFloat16` stops adding at `k = 23` (A). Show the program, trimmed, and the table.
4. (Across crates) The same alias through tensor, topology, haft and multivector: residuals move twenty-five orders of magnitude together, and nothing was converted between crates because there was nothing to convert (A). Show the residual table. Row P3 closes here: the drift has an oracle now.
5. (When it pays) Drift widens only for chained transcendental work; default to `f64` and reach for more when the structure amplifies rounding (C).
6. (Then) Mixed precision falls out: three parts, three limits, a budget each, and a program that picks the narrowest precision meeting each budget and asserts the pick (A). Show the pick table. `Float106` at two to four times an `f64` operation is spent on reductions and never on fields (A).
7. (Priced) On the forecast ensemble, `2182 GB` of state becomes `1299 GB`, the nodes drop one rung, and the run costs `$11.72` where it cost `$22.62`; the `24 %` on the rate is list-price arithmetic and the `32 %` on the clock is ECMWF's figure carried over (A, I13). Row P7 closes here.

**Closing sentence.** The alias costs one line, the lifts are written once, and everything the
program earns from them, thirty digits or half the memory, it earns without a second copy of the
math.

**Transition.** That leaves the eight rows to read again.

**Code to carry.** The alias and scalar table (A); the telescoping program and digits table (A);
the cross-crate residual table (A); the pick table and six rows of the price table (A).

**Verbs.** names, fixes, downcasts, lifts, fail, stops, move, converted, widens, picks, asserts,
spent, drops, costs, earns.

### 4. Close: the eight rows, revisited

**Main point.** Seven of the eight translations have nothing left to translate, one is kept on
purpose, and the reader can check every claim in three commands.

**Topic sentence.** Read the eight rows again with the stack in hand.

**Paragraphs.** 3: the rows, one line each; what stays; the invitation.

**Middle, in order.**
1. (Row by row) P1 and P6 by the witness and nesting; P2 by `metric`; P3 by the alias and the residual tables; P4 by law tests at the witness and by bounds; P5 by `stats` and `rand`; P7 by the pick; P8 by `CausalFlow`. Show a two-column table, row and answer.
2. (However) What stays: the outer edge of the stack, where a `BFloat16` slice meets an accelerator's buffer and is kept as the one bridge; the `none` row; the cost of `Float106` (A). State each once.
3. (For the reader) Default to `f64`; measure against a closed form; say what was measured and what was assumed (C, A).
4. (Invitation) The three commands (A).

**Closing sentence.** Ten witnesses, one alias, and three commands: the translations have an owner
now, and the reader can run the proof before lunch.

**Code to carry.** The rows-revisited table (H, A, B); the three `cargo run` lines (A).

**Verbs.** read, close, keep, stay, default, measure, say, run.

---

## Part 3: The three Aristotelian rules, applied

**Logos: premise, deduction, conclusion.**
- Premise 1: every pain point in a cross-domain simulation is one of five things translated by hand at a call site: a container, a convention, a scalar, a law, or a primitive.
- Premise 2: a hand translation has no owner when it fails and no line item when it costs.
- Evidence: faults cluster at interfaces, one in seven in Fortran (I1); the sign flip that retracted five papers crossed a lab boundary in a supplied program (I2, I3); the Mars Climate Orbiter's units crossed a software interface (I10); agreement fell from six significant figures to one with no owner (I1); the single-precision saving at ECMWF was a line item only once measured (I13).
- Deduction: move each translation into the one place that owns the container, give every container the same verbs, and make the scalar a parameter; then the call site holds no glue, the pattern transfers across crates, and the program reruns at any precision from one edit.
- Conclusion: ten witnesses against ninety bridges; six steps in one `extend`; thirty digits from one line; `$11.72` against `$22.62`.
- Each section ends on the premise of the next; the close returns to the eight rows.

**Ethos: credibility carried by evidence.**
- The assessment is labelled as the author's, every row carries a citation, and the reader is invited to strike rows.
- The witness section shows the implementation, the law-test names, and the docstring that records the corner the laws cannot close.
- Every number cites the program that printed it and the scalar it ran at; any wall-clock figure names the machine.
- Each removal gets one sentence with the failing input.
- The estimate section keeps its first sentence: nothing in it was measured.

**Pathos: undivided attention to one reader.**
- Open on the reader's own afternoon: the plumbing, the sign that passed, the drift with no owner.
- Keep the tone of someone who found a defect and fixed it, never of someone selling; the withdrawn `Monad` and the docstring caveat carry that tone.
- Close with an invitation the reader can act on in one minute: three commands.
- The price table speaks to the reader who signs for the nodes.

---

## Part 4: The verb check, before each paragraph is written

Weak verbs to strike on sight: is, are, has, provides, allows, enables, supports, leverages,
offers, features, utilizes, handles.

Strong verbs the material supplies: pay, translate, flatten, convert, bind, implement, test, find,
fix, withdraw, record, rotate, nest, reach, replace, carry, admit, walk, contract, cross, name,
serve, agree, lift, stop, move, widen, pick, assert, spend, drop, cost, earn, measure, run.

Per paragraph, the four questions from the guide: strong or weak; does the chain tell the story;
do the verbs point back at the main point; any repeats.

## Part 5: House rules for the draft

- No em-dashes; no "not A, it's B"; no uniqueness claims; state, do not argue.
- The pain-point assessment names symptoms and roots; it names no other library.
- Real code or real output in every section, cut from `main`; the witness impl and the law tests are quoted from source, not paraphrased.
- `FloatType` alias in every snippet; `f64` only at the display boundary.
- Say "project website" where the website comes up.
- Sentence length varies; the connector matches the relation it signals.
- Six revision passes from `ElementsOfStyle.md`: cut, verb, order, cohesion, voice, reader.
