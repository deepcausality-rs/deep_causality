<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Example Design Note: `geometric_qec`

**What this is.** The design for a third example under `examples/quantum_examples/`, covering the
**gates avenue** of [`positioning.md`](positioning.md) §4.2, which currently has no example at all.
Companion to [`example-quantum-control-loop.md`](example-quantum-control-loop.md),
[`example-crosstalk-attribution.md`](example-crosstalk-attribution.md) and
[`qcl-dsl-liftback.md`](qcl-dsl-liftback.md).

**Status.** Design only. Every table below was computed by running the crates, not recalled.

**The thesis:**

> A quantum code is a shape. Its logical qubits are holes in that shape, its parity checks are the
> shape's boundary maps, and its logical gates act on homology classes rather than on operators.
> Change the shape and the code changes with it.

---

## 1. What already ships, and why this construction

### 1.1 The surprise: most of this already ships

I expected this example to be blocked on the topology track, since
[`dynamic-qcm.md`](dynamic-qcm.md) §3.3 listed cup products as the keystone gap. Running the crates
says otherwise. `deep_causality_topology` carries `ChainComplex` with `boundary_matrix(k)`,
`coboundary_matrix(k)` and `betti_number(k)`, over `CellComplex`, `SimplicialComplex` and
`LatticeComplex`, plus a full lattice gauge theory module with link variables and Wilson loops.

Those are precisely the objects a CSS code is made of. A CSS code **is** a chain complex; the
boundary maps **are** the parity-check matrices; the logical qubits **are** counted by `β₁`.

Running `LatticeComplex::<2, f64>::square_torus(L)` and reading its invariants:

| L | vertices | edges | faces | β₀ | β₁ | β₂ | code |
|---:|---:|---:|---:|---:|---:|---:|---|
| 2 | 4 | 8 | 4 | 1 | **2** | 1 | [[8, 2, 2]] |
| 3 | 9 | 18 | 9 | 1 | **2** | 1 | [[18, 2, 3]] |
| 4 | 16 | 32 | 16 | 1 | **2** | 1 | [[32, 2, 4]] |
| 5 | 25 | 50 | 25 | 1 | **2** | 1 | [[50, 2, 5]] |
| 6 | 36 | 72 | 36 | 1 | **2** | 1 | [[72, 2, 6]] |
| 8 | 64 | 128 | 64 | 1 | **2** | 1 | [[128, 2, 8]] |

That is the standard toric code family, `n = 2L²` physical qubits and `k = 2` logical qubits,
produced from a periodic lattice with no code-specific code written. The parity checks come out as
sparse matrices:

```
L=4:  ∂₁ (vertices × edges) shape (16, 32)  nnz 64   ->  H_X
      ∂₂ (edges × faces)    shape (32, 16)  nnz 64   ->  H_Z^T
      H_X: average row weight 4.0, maximum column weight 2
      H_Z: average row weight 4.0, maximum column weight 4
```

Weight-4 checks with bounded column weight, which is the **LDPC property**, read directly off the
`CsrMatrix<i8>` structure `deep_causality_sparse` already provides. Sparsity is not an
implementation detail here; bounded check weight is part of the definition of a qLDPC code, and the
representation carries it.

The Betti vector generalises. A 3-torus at L = 3 returns `[1, 3, 3, 1]`: three logical qubits, with
`β₁ = β₂` by Poincaré duality. That duality is the beginning of the gate-catalogue story in
`dynamic-qcm.md` §3.2, where addressable `C^{n−q−1}Z` generators are counted by `b_q = b_{n−q}`. The
**counting** is available today. The gates themselves are not; see §6.

One honesty note on the table above: `n` and `k` were computed by the crate. **`d` was not.** The
distance of the L × L toric code is L by a standard argument, and computing distance in general is
the minimum-weight homology representative problem, which is NP-hard. The example should print `d`
for small complexes by enumeration and say plainly that it is doing so.

---

### 1.2 Why the Haruna line: generality over CSS codes

The gate list is not what makes this construction the right one to build on. The relaxation is.

Hsin–Kobayashi–Zhu solve two problems at once: what logical action a code supports, and how to
realize it at constant depth without spreading faults. Solving both together costs generality,
because the constant-depth argument needs geometric structure and applies where that structure
exists.

Haruna drops the circuit constraint and gets **general CSS codes**: no manifold, no product
structure, no locality requirement. The logical action of `S`, `H`, `T` and `CZ` becomes computable
for any CSS code from its chain complex alone.

That matches where codes are going. Surface codes have encoding rate `k/n → 0`, and the move to
qLDPC is a move to constant rate. qLDPC codes are built from products of chain complexes, lifted
products and expander constructions; they carry arbitrary structure and no geometry to lean on. A
construction requiring geometric structure does not reach that regime; one over general CSS codes
does.

The ordering is forced. The logical action is the specification any correct circuit must realize, so
it has to be computable before circuit synthesis is well-posed, and it has to be computable for the
code in hand rather than for codes whose geometry cooperates.

**The design consequence.** The cup product must be generic over `ChainComplex`. The torus is a test
case; arbitrary complexes are the target. A cup product specialised to lattices would reproduce the
toric code and reach nothing past it. This is written into `openspec/changes/archive/2026-08-20-add-cup-product` as a
requirement rather than left as an intention.

**Why it is worth doing.** If the native gate set of an arbitrary CSS code is computable, then
searching over candidate complexes for a code whose gate set serves a given algorithm is a search
with a computable objective. That is the co-design direction of `dynamic-qcm.md` §3.5, and it needs
generality over codes rather than a longer gate list on one code.

---

## 2. Logical operators are homology classes

A logical `Z` on the toric code is a cycle of edges wrapping the torus and not bounding any region.
A logical `X` is the dual object on the dual lattice. Two independent wrapping directions give
`β₁ = 2` logical qubits, which is the same 2 the crate just computed.

The part that matters for the example: **two cycles in the same homology class differ by a
boundary.** Given a cycle `γ`, adding `∂f` for any face `f` produces `γ' = γ + ∂f`, a different set
of physical edges representing the same logical operator. In code language, multiplying a logical
operator by a stabilizer leaves the logical action unchanged.

That is generated directly from `boundary_matrix(2)`, so the example can produce as many
representatives of a class as it likes without hand-drawing cycles.

---

## 3. The validate gate: homology-class invariance

Haruna's construction (arXiv:2511.15224) writes logical gates as exponentials of gauge-field
polynomials. The crate ships them with exactly this shape:

```
logical_z(a_γ)          = exp( i π a(γ) )
logical_x(b_γ̃)          = exp( i π b(γ̃) )
logical_s(a_γ)          = exp( i (π/2) a(γ)² )
logical_hadamard(a_γ, b_γ̃)
logical_cz, logical_t
```

The argument is the gauge field integrated over a cycle. Different representatives of one homology
class give **different** `a(γ)`. The theorem Haruna proves is that the resulting logical action
depends only on the class.

That theorem is a check, and this codebase turns checks into freeze-time gates. The example makes it
the **third structural gate**, alongside the quantum Markov commutativity check and C₃-exclusion
faithfulness:

> Generate several representatives `γ, γ + ∂f₁, γ + ∂f₂, …` of one homology class. Apply the gate to
> each. The logical actions must agree to tolerance. If they do not, the construction has been
> misapplied to this complex, and the freeze aborts naming the pair of representatives that
> disagreed.

This is the example's core, it is buildable today, and it is a genuine correctness criterion rather
than a demonstration of a language feature. It also exercises the gates' typed failure: a field that
overflows the guarded Taylor series surfaces `QuantumError` instead of a silent identity, which
matters exactly here, because a silent identity would pass an invariance check trivially.

---

## 4. Where the DSL applies, and where it does not

This is the most useful thing this note can report, and it is not uniformly positive.

**Validate: strong fit.** The pipeline gains a third gate and, more importantly, a third *kind* of
subject. Until now `validate` took a factorization. Here it takes a **code**. That generalisation is
what the example contributes back to the QCL sketch: the validate pipeline is the home for
structural correctness criteria in general, not for QCM-specific ones.

```rust
QclBuilder::build_validate(&cfg)
    .declare_complex()          // the cell complex
    .derive_code()              // n from 1-cells, k from β₁, checks from ∂₁ and ∂₂
    .check_ldpc_weights()       // bounded row and column weight
    .check_class_invariance()   // §3: the Haruna gate acts on the class, not the representative
    .validate_analyze()
    .finalize()
    .print_results();
```

**Design: partial fit, and honestly bounded.** There is a real selection problem, which is choosing
a complex that delivers a required `k` and `d` and gate set at the lowest physical qubit count. It
has the shape of §8.8's minimum-cost cover, with the requirement set as elements and candidate
complexes as covering options. Three of the four axes are computable today: `n` and `k` exactly, `d`
by enumeration on small complexes, and — since `SPEC-T2` shipped — the individual `CZ`-class gates a
complex supports, by evaluating the cup product on its cocycles. What is still missing is the
**catalogue**: the full inventory of independently addressable generators, which needs the higher
cup products of `SPEC-T3` and the duality bookkeeping that goes with them. The example should run
the design over `(n, k, d)` plus the gates it can actually evaluate, and name the catalogue as the
remaining gap rather than faking it.

**Control: no fit, and the example should say so.** There is no monitor, no envelope, no
intervention here. This is construction and verification. The natural control loop over a code is
decoding, and decoding is Track D, which `positioning.md` commits to claiming nothing about until
SPEC-D1 produces a number. Forcing a control pipeline onto this example would be the same category
error as putting a deontic gate on a decoherence refusal.

That the DSL's two pipelines are not equally applicable everywhere is a finding worth writing down.
A framework that claims uniform applicability is one nobody in this audience believes.

---

## 5. What each example has contributed back

Three examples, three amendments, which is the pattern a design should want:

| Example | What it forced |
|---|---|
| `quantum_control_loop` | the pipeline shape: prepare, observe, gate, fork, predict, adjudicate |
| `crosstalk_attribution` | `design` returns a **plan**, not an experiment (minimum-cost cover, §8.8) |
| `geometric_qec` | `validate` takes a **code**, not only a factorization; a third structural gate |

---

## 6. The gap, now closed at the cup-product rung

**Status, 2026-08-21.** Items 1 through 4 of §6.3 are delivered
(`openspec/changes/archive/2026-08-20-add-cup-product`). `deep_causality_topology` now carries the binary and `n`-fold
cup product, generic over `ChainComplex`, for both the simplicial and cubical families. What remains
of this section is the reasoning that produced that scope and the rungs still above it, kept because
the estimate it corrected is worth remembering.

Everything above runs on what ships. The next rung was the cup product, and checking the data
structures changed my estimate of what it costs.

### 6.1 Why a cup product needs an ordering at all

On simplicial cochains the cup product is the Alexander–Whitney formula. For `α` of degree `p`, `β`
of degree `q`, and a `(p+q)`-simplex `σ = [v₀, …, v_{p+q}]`:

```
(α ∪ β)(σ) = α([v₀, …, v_p]) · β([v_p, …, v_{p+q}])
```

The formula splits the simplex into a **front `p`-face** and a **back `q`-face**. Front and back are
meaningless unless the vertices of `σ` are ordered, and the orderings on overlapping simplices must
agree or the products will not assemble into a cochain. A global vertex ordering supplies exactly
that, and in the triangulated-manifold literature it is what "branching structure" names. The
cubical analogue is Serre's formula, which splits a cell's direction set into a front subset and a
back subset, and needs the same thing: an order on the directions and hence on the cell's corners.

There is a second fact that matters more than it looks. The cochain-level cup product **depends on
the ordering**; the induced map on cohomology **does not**, because two orderings give products
differing by a coboundary. That is precisely why Haruna's logical gates are well-defined on
homology classes rather than on representatives, and it is the same invariance §3 checks. In the
formalism's own language, the branching structure is a **gauge choice**: the formula needs one, the
physics does not depend on which.

### 6.2 The ordering is already there

`dynamic-qcm.md` §3.3 lists `SPEC-T1`, a global vertex ordering, as the keystone that gates
everything. Reading the types says it is already satisfied by construction, in both complex
families:

- **Simplicial.** `Simplex::new` calls `vertices.sort_unstable()` and stores a sorted `Vec<usize>`.
  That is a global vertex order, induced from the order on `usize` and consistent across every
  simplex. `Simplex::subsimplex(range)` slices that sorted list, which is exactly the front-face and
  back-face extraction Alexander–Whitney asks for.
- **Cubical.** `LatticeCell<D> { position: [usize; D], orientation: u32 }` names a cell by its base
  corner and the bitmask of axes it extends along. The axis order and the global coordinate order
  together order every cell's `2^k` corners canonically, consistently across cells.

So the keystone is not missing. It is present, unnamed, and load-bearing by accident.

### 6.3 What actually has to be added

Not needed, because it exists: a global vertex ordering, and front/back face extraction on
simplices.

Needed, in dependency order:

1. **Promote the ordering to a contract.** Today it is an implementation detail: `Simplex.vertices`
   is `pub(crate)` and nothing documents that sortedness is guaranteed. A cup product that silently
   depends on an unstated invariant is a bug waiting for a refactor. A `Cell` trait method exposing
   the ordered vertices or corners, with the invariant written down, is the real `SPEC-T1` and it is
   small.
2. **A cochain type** carrying the dual of `Chain<T>`, if one does not already exist.
3. **The Alexander–Whitney cup product** on simplicial cochains. Given `subsimplex`, this is a few
   lines.
4. **Serre's cubical cup product**: sum over splittings of a cell's direction set into a front part
   and a back part, with the shuffle sign. Fully determined by `(position, orientation)`.
5. **Sign consistency against the existing boundary operators.** The acceptance test is the Leibniz
   rule `δ(α ∪ β) = δα ∪ β ± α ∪ δβ`. It is the one place the cubical signs can be wrong in a way
   that is invisible until a gate misbehaves, and it should be a property test rather than a
   spot check.
6. **Higher cup products `∪ᵢ`** by Steenrod's recursion, then `Sqⁱ(α) = α ∪_{n−i} α`. These are what
   the Hsin–Kobayashi–Zhu higher gates need; Haruna's `CZ` needs only step 3 or 4.

Every one of these is written against `ChainComplex` rather than against a concrete complex, for the
reason in §1.2. Specialising to `LatticeComplex` would be simpler and would forfeit the only property
that makes the work worth doing.

Items 1 through 4 are the whole distance between what ran before and a working logical `CZ` on the
toric code, **and all four are now done**. Item 2 resolved itself: no `Cochain` type was added,
because the repository's convention is a flat slice indexed by cell index and a wrapper would have
forced conversions on existing callers. Items 5 and 6 remain, item 6 being the gate on the
catalogue. That is a smaller gap than the ladder in `dynamic-qcm.md` implies, and the reason it
looked larger is that the ladder was written against a specification rather than against the types.

### 6.4 What stays out of reach

**What the example can now build.** The cup product is available, so a logical `CZ` on the toric
code and a `CCZ` on a 3-torus can be constructed from it, and their homology-class invariance
checked. The crate delivers the *operation*, not the gate: `deep_causality_quantum` does not depend
on `deep_causality_topology`, so the example itself is where the two meet, and no fault-tolerance
claim attaches either way (§11 items 1 and 5).

The gate **catalogue** still lies out of reach, and with it the code search that consumes it. Counting
independently addressable `C^{n−q−1}Z` generators from the Betti vector needs the higher products of
item 6 and the duality bookkeeping that goes with them. The dependency runs catalogue first, search
second: a search over complexes needs a computable objective, and the objective is the native gate
set. The example can report that a 3-torus encodes three logical qubits and that `β₁ = β₂ = 3`; it
cannot yet say which multi-controlled gates that duality makes addressable. A working
`geometric_qec` stopping one clearly-named rung short is a better argument for building items 1
through 4 than the specification is.

## 7. The crate ecosystem

| Crate | Contribution |
|---|---|
| `deep_causality_topology` | `LatticeComplex` and `ChainComplex`: cells, `boundary_matrix`, `coboundary_matrix`, `betti_number`; the gauge field lattice, link variables and Wilson loops |
| `deep_causality_sparse` | `CsrMatrix<i8>` carries the parity checks; the LDPC property is read off its structure |
| `deep_causality_quantum` | the six Haruna logical gates, with overflow and non-convergence as typed errors |
| `deep_causality_multivector` | `CausalMultiVector` as the gauge field carrier the gates consume |
| `deep_causality_num_complex` | `Complex` coefficients |
| `deep_causality_core` | the pipeline and the audit trail |
| `deep_causality_algebra` | `RealField`; the verdict lattice the invariance check reports into |

Two of these were flagged during review and both proved load-bearing. The topology crate supplies
far more geometric machinery than the gap inventory suggested, and the sparse crate is not a
storage detail but the representation in which the defining property of a qLDPC code is visible.

---

## 8. DEM interop: the ecosystem's format is already a causal model

Stim's Detector Error Model is the practical interchange standard across the QEC simulation
ecosystem. Reading and writing it puts this example upstream of Stim and PyMatching rather than
beside them.

### 8.1 The format

Five instructions, UTF-8, double-precision coordinates:

```
error(0.001) D0 D1 L0                 # probability, then symptoms and frame changes
error(0.02) D2 L0 ^ D5 D6             # ^ suggests a decomposition into edge-like parts
detector(2.5, 3.5, 6) D7              # coordinates, relative to the running offset
logical_observable L1 L2              # assert the frame change exists
shift_detectors(0, 0.5) 2             # advance detector index by 2, y by 0.5
repeat 9 { ... }                      # a time-homogeneous block
error[tag_name](0.1) D0               # optional tag
```

A DEM is a list of **independent** error mechanisms. Each carries a probability, the set of detectors
it flips, and the set of logical observables it flips. Sampling keeps each mechanism independently
and XORs the results, so symptoms appearing an odd number of times survive.

Three details decide whether an importer is correct:

1. **`D#` is relative, `L#` is absolute.** Detector indices are adjusted by the running offset;
   observable indices are not. Treating them alike mis-wires every `repeat` block.
2. **`repeat` with `shift_detectors` encodes time.** A repeat block is a time-homogeneous causal
   process and the shift is the temporal stride. Preserve it; expanding it discards the stationarity
   that makes the model compact.
3. **`^` is a decoder hint.** It records a suggested decomposition for matching decoders and carries
   no causal content. Drop it on import, regenerate it on export.

### 8.2 The correspondence is exact

A DEM is a bipartite graph from latent mechanisms to observed symptoms, with independent mechanisms
and known marginals. `error(0.001) D0 D1 L0` reads directly as a hypothesis: a mechanism with effect
set `{D0, D1, L0}` at `p = 0.001`. No translation layer is required.

### 8.3 A DEM imports into two different configs

This is where the subject-keyed config of [`qcl-design-note.md`](qcl-design-note.md) §4.2 earns its
keep. A DEM carries a code **and** a noise model, and the two answer different questions.

**Structural questions — `over_code`.** No device, exact evaluation.

```rust
let cfg = QclBuilder::config::<FloatType>()
    .over_code(Code::from_dem_file("surface_d5.dem")?)
    .build()?;

QclBuilder::validate(&cfg)
    .check_degeneracy()
    .check_ldpc_weights()
    .finalize().print_results();
```

**Diagnostic questions — `over_plant`.** The DEM supplies the hypothesis structure; the device
supplies the evidence.

```rust
let dem = Dem::from_file("surface_d5.dem")?;

let cfg = QclBuilder::config::<FloatType>()
    .over_plant(device)
    .evidence(Evidence::shots(4096).seed(20260821))
    .baseline(Experiment::probe("syndrome", detector_readout, 1, cost = 1))
    .probes(&targeted_experiments)
    .candidates(&dem.mechanisms())          // each error(p) line becomes a hypothesis
    .build()?;
```

### 8.4 The check that does not exist today

Group error mechanisms by identical target set. Any group with more than one member is a
**Markov-equivalence class: indistinguishable from syndrome data alone**.

Stim combines such mechanisms by design, and for decoding they are the same — the correction depends
on the symptom pattern, not on which fault produced it. For diagnosis they are different physical
faults with different repairs, and the merge removes the only signal that would separate them. This
is the failure class the calibration example opens on: a confident answer with nothing marking it as
uninformative.

`check_degeneracy` reports the class and names the experiment that breaks it. That is `validate`
applied to a DEM, and it is the most defensible thing this line of work adds at the QEC tier.

### 8.5 Export

The derived code plus a noise model emits DEM text, which Stim samples and PyMatching decodes:

```rust
QclBuilder::validate(&cfg)
    .derive_code()
    .check_class_invariance()
    .finalize()
    .to_dem(&noise)?;
```

### 8.6 What this must not claim

The example does not decode. Stim and PyMatching decode, and they are fast and correct at it. The
contribution is upstream — deriving the code from a complex, checking that logical gates act on
homology classes rather than representatives, and reporting which error mechanisms syndrome data
cannot separate. Two recent lines of work sit nearby and should be cited rather than ignored: DEM
reconstruction from syndrome data (arXiv 2606.16288, 2601.22286), which estimates parameters over a
known structure, and quasilinear DEM equivalence checking (arXiv 2606.14677), which verifies rather
than attributes.

---

## 9. Sketch of the output

```
=== Geometric quantum error correction: codes from shapes ===

[complex] LatticeComplex<2> square torus, L = 4
    cells: 16 vertices, 32 edges, 16 faces
    Betti: b0 = 1, b1 = 2, b2 = 1

[code] derived from the boundary maps
    n = 32 physical qubits  (1-cells)
    k = 2 logical qubits    (b1)
    d = 4                   (minimum-weight representative, by enumeration)
    H_X = d1, shape (16, 32), row weight 4, column weight 2
    H_Z = d2^T, shape (16, 32), row weight 4, column weight 2
    LDPC: bounded on both axes, ok

[validate] homology-class invariance of the Haruna gates
    class [gamma_1], 6 representatives generated as gamma + boundary
      logical_z : max pairwise action difference 3.1e-16   ok
      logical_s : max pairwise action difference 5.7e-16   ok
      logical_h : max pairwise action difference 8.2e-16   ok
    class [gamma_2], 6 representatives
      logical_z : max pairwise action difference 2.8e-16   ok
    -> the gate acts on the class, not on the representative (Haruna 2025)

[design] cheapest complex delivering k >= 2, d >= 5
    2-torus L=5   n =  50   k = 2   d = 5   ok      <- selected
    2-torus L=6   n =  72   k = 2   d = 6   ok
    3-torus L=3   n =  81   k = 3   d = 3   distance short
    gate-catalogue axis not evaluated: cup products unavailable (SPEC-T1/T2)

[not run] control pipeline. There is no loop here; decoding is Track D.
```

Numbers in the complex and code blocks are computed. The invariance residuals are illustrative
until the example runs.

---

## 10. Layout and build

```
examples/quantum_examples/geometric_qec/
  README.md  main.rs  constants.rs  model_config.rs  model_types.rs  model.rs  utils_print.rs
```

Registration in `Cargo.toml`, a `rust_binary` in `BUILD.bazel` with topology, sparse, quantum,
multivector, num_complex and core in `deps`, and a row in the package README table. Per-example
`FloatType` alias, `f64` only at the display boundary.

---

## 11. What the example must not claim

1. **The gates are not fault-tolerant.** Haruna states his focus "is not on fault tolerance such as
   constant depth or locality." The constant-depth line is Hsin–Kobayashi–Zhu and is not
   implemented. This is `positioning.md` boundary 10 and belongs on the same page as the gates.
2. **Logical gates are not algorithms.** A gate set is an instruction set. Compiling an algorithm
   onto it is a separate discipline.
3. **`d` is enumerated, not solved.** Distance is computed by brute force on small complexes.
   Minimum-weight homology representative is NP-hard in general.
4. **The gate catalogue is absent.** The example counts logical qubits from `β₁` and can evaluate
   individual `CZ`-class gates now that the cup product ships, but it does not enumerate the
   *inventory* of independently addressable generators. That needs the higher cup products of
   `SPEC-T3` and the duality bookkeeping with them.
5. **No decoder, no noise, no threshold.** Nothing here is a statement about logical error rates.
6. **The toric code is a demonstration, not a contribution.** Its parameters are textbook. What the
   example shows is that they fall out of a general complex rather than out of code-specific
   software.

---

## 12. Open questions

1. **Which complex family?** The 2-torus is the clearest. Adding the 3-torus shows the Betti vector
   generalising and costs a few lines. *Recommendation: both.*
2. **Does the example demonstrate genericity, or only assert it?** Both tori are lattice complexes,
   so an example built only on them exercises one `ChainComplex` implementor and proves nothing about
   the property §1.2 says is the point. *Recommendation: include at least one non-lattice complex, a
   `SimplicialComplex` built by hand, so the genericity is executed rather than claimed.*
3. **Does the invariance check ship as a crate function or stay in the example?** It is a genuine
   structural gate with a theorem behind it. *Recommendation: prototype in the example, and lift it
   to the crate once the third example proves the shape, on the same rule the composites in
   `qcl-dsl-liftback.md` §7 follow.*
4. **Should a deliberately failing case be included?** A complex where the construction is misapplied
   and the invariance check aborts would do for this gate what the non-commuting model does for
   `qcm_freeze_check`. *Recommendation: yes, if a physically honest failing case can be found; do not
   manufacture one.*
5. ~~**Does this example justify starting `SPEC-T1` and `T2`?**~~ **Answered and done.** §6.2 was
   right that the ordering was already present, so `SPEC-T1` reduced to documenting the invariant.
   Both are delivered and `dynamic-qcm.md` is re-scoped. The open question that replaces it: does the
   example get written next, or does `SPEC-T3` (higher cup products) come first? *Recommendation:
   the example, since it now has everything it needs and would exercise the new operation against a
   real code.*

---

## 13. Sources

- Haruna, J. (2025). *Note on Logical Gates by Gauge Field Formalism of Quantum Error Correction.*
  arXiv:2511.15224. The gate construction and the homology-class invariance theorem the validate
  gate checks. Bundled in `deep_causality_quantum/papers/`.
- Hsin, P.-S., Kobayashi, R. & Zhu, G. (2024). arXiv:2411.15848. The fault-tolerant constant-depth
  line, cited as the boundary of what is implemented, not as what is implemented.
- Gidney, C. (2021). *Stim: a fast stabilizer circuit simulator.* Quantum **5**, 497. And
  `doc/file_format_dem_detector_error_model.md` in `quantumlib/Stim`, the Detector Error Model
  specification §8.1 follows: five instructions, relative `D#` against absolute `L#`, `repeat` with
  `shift_detectors` for time-homogeneous blocks, `^` as a decoder hint.
- DEM reconstruction from syndrome data, arXiv:2606.16288 and arXiv:2601.22286. Parameter estimation
  over a known structure, adjacent to §8.4.
- *Quasilinear Equivalence Checking for Detector Error Models*, arXiv:2606.14677. Verification, not
  causal attribution.
- `dynamic-qcm.md` §3, for the QEC-as-topology identification and the `SPEC-T1` through `SPEC-T6`
  ladder this example stops against.

Verified against the crates on 2026-08-20: `LatticeComplex::square_torus`, `ChainComplex::{cells,
num_cells, boundary_matrix, betti_number}`, `CsrMatrix::{shape, col_indices, values}`, and the
`logical_*` signatures in `deep_causality_quantum/src/types/qgates/gates_haruna.rs`.

Companion notes: [`positioning.md`](positioning.md),
[`example-quantum-control-loop.md`](example-quantum-control-loop.md),
[`example-crosstalk-attribution.md`](example-crosstalk-attribution.md),
[`qcl-dsl-liftback.md`](qcl-dsl-liftback.md).
