## Why

Cohomology-operation logical gates for CSS codes are exponentials of cup products of gauge-field
cochains (Haruna, arXiv:2511.15224; Hsin–Kobayashi–Zhu, arXiv:2411.15848). `deep_causality_quantum`
already ships the Haruna logical gates, and `deep_causality_topology` already supplies everything
they sit on: chain complexes, boundary and coboundary matrices, Betti numbers, and cell and lattice
complexes. Running the crate confirms an `L × L` periodic `LatticeComplex` reproduces the toric code
family with `β₁ = 2` and weight-4 parity checks, with no code-specific code written.

The one missing operation is the cup product itself. Without it a logical `CZ` cannot be constructed,
and the `geometric_qec` example stops one rung short of the gate catalogue.

The prior roadmap (`openspec/notes/quantum/dynamic-qcm.md` §3.3) identified a global vertex ordering
("branching structure") as the keystone blocker. Investigation of the code shows it is already
present in both complex families, so the gap is smaller and differently shaped than recorded.

## What Changes

- Add a **cell-splitting abstraction** as a new trait, distinct from `Cell`. Splitting, not vertex
  listing, is the shared operation: a simplex has exactly one left/right splitting per left degree
  (Alexander–Whitney is a single term), while a `k`-cube has `C(k, p)` splittings carrying shuffle
  signs (Serre's formula is a sum). A simplex's vertices are `usize` ids and a lattice cell's are
  `[usize; D]` positions, so no common vertex type is workable, but a common splitting is.
- Implement the trait for `Simplex` and `LatticeCell<D>`.
- Add a **degree-general cup product** over cochains, generic across both families via the trait.
- Add the **`n`-fold cup product**, a fold of the binary product over a slice of cochains, and with
  it the multi-controlled `C^{n−1}Z` logical actions on complexes of dimension `n`.
- **Document the ordering invariants that already hold**: that `Simplex::vertices()` returns a
  strictly increasing list (already stated on the type, absent from the accessor), and the corner
  enumeration order of `LatticeCell::vertices()` (deterministic today, undocumented).
- **Relocate the existing Alexander–Whitney implementation.**
  `deep_causality_physics::kernels::mhd::ideal::wedge_product_1form_1form` already implements the AW
  formula at fixed degree `(1,1)`. It becomes a caller of the general topology implementation.
- Add the **Leibniz property test** against the crate's existing boundary and coboundary operators.

No breaking changes. Every element is additive: a new trait, new free functions, and doc-only edits.
No existing signature is modified.

The `n`-fold product is included because the cup product is associative, so `CCZ` via a triple
product and `C^{n−1}Z` via an `n`-fold product need no machinery beyond the binary case
(`openspec/notes/quantum/dynamic-qcm.md` §3.2 states exactly this decomposition). Its marginal cost
is an `n`-ary wrapper and its tests, while its gain is the whole non-Clifford multi-controlled
family rather than `CZ` alone. `LatticeComplex::<3, R>::cubic_torus` already provides the
three-dimensional complex a triple product needs.

**Explicitly out of scope**, and left for a follow-up change: higher cup products (`∪ᵢ`), Steenrod
squares, higher Pontryagin powers, addressability via higher-form symmetries, and the Betti-vector
gate catalogue. Those are a materially larger piece of work (`dynamic-qcm.md` §3.2 line 2 and
`SPEC-T3`) whose payoff only lands once addressability and the catalogue land with them. The cost
and gain of each tier is recorded in `design.md`.

**Also out of scope**: emitting fault-tolerant physical-gate decompositions. The cup product yields a
logical *action* and its homology-invariance criterion. The constant-depth circuits that realise
those actions are a separate, larger piece, and no fault-tolerance claim is made by this change.

## Capabilities

### New Capabilities

- `cell-splitting`: the front/back splitting of a cell with sign, as a trait separate from `Cell`,
  implemented for `Simplex` and `LatticeCell<D>`, together with the documented vertex and corner
  ordering invariants the splittings rest on.
- `cup-product`: the degree-general binary and `n`-fold cup product on cochains over any complex
  whose cells split, covering the simplicial (Alexander–Whitney) and cubical (Serre) instances, and
  the algebraic laws that make it correct: associativity, Leibniz against the existing coboundary,
  and graded commutativity up to a coboundary.

### Modified Capabilities

None. The physics relocation is an implementation change behind a private function with a single
caller, and alters no requirement.

## Impact

**Crates that depend on `deep_causality_topology`**: `deep_causality_physics`,
`deep_causality_cfd`, `deep_causality_algorithms`, `deep_causality_discovery`, and five example
packages. All are unaffected, because the change adds surface rather than altering it.

**Measured blast radius of the surfaces this change touches or builds on** (non-test callsites):

| Surface | Non-test callsites | Consequence for this change |
|---|---:|---|
| `Cell` trait implementors | 3, all in-crate | A required method on `Cell` would break external implementors. Use a separate trait. |
| `: Cell` as a bound | 132 | Confirms `Cell` is load-bearing and must not change. |
| `Simplex::new` | 47 | Signature must not change. Unaffected. |
| `Simplex::subsimplex` | **0** | Public, unused outside tests, and exactly the AW extractor. Safe to build on. |
| `LatticeCell::*` | 41 | Read-only accessors, unaffected. |
| `Skeleton::simplices` | 22 | Read-only, unaffected. |
| `wedge_product_1form_1form` | 1, private | Relocation is invisible outside `deep_causality_physics`. |

**Regression risk** concentrates in one place: the relocated Alexander–Whitney implementation must
be numerically identical, since it feeds the MHD ideal-induction kernel. The existing kernel tests
are the guard.

**Downstream unblocked**: the `geometric_qec` example
(`openspec/notes/quantum/example-geometric-qec.md`), a logical `CZ` on the toric code, and the
multi-controlled `C^{n−1}Z` family on higher-dimensional complexes.

**Roadmap correction**: `openspec/notes/quantum/dynamic-qcm.md` §3.3 lists `SPEC-T1` (branching
structure) as an unbuilt keystone. It should be re-scoped to what this change actually needs, which
is documenting an invariant that already holds rather than constructing one.
