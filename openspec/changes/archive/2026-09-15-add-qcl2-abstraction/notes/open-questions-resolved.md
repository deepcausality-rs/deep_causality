<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# The three open questions of `design.md`, investigated

Investigated 2026-09-09 against `deep_causality_topology` and `deep_causality_quantum` at the
working tree, with a scratch probe crate built against the workspace, `numpy` 2.4.4 for the
operator checks, and Haruna, arXiv:2511.15224 (`deep_causality_quantum/papers/
logical_quantum_gates_by_gauge_field_formalism.pdf`) for the gate algebra. Each section states the
question as the design has it, what was measured or derived, and what the design should now say.
Nothing in `design.md`, the specs or the tasks has been edited; the edits are listed at the end.

## 1. `square_torus(2)` is a valid complex and gives `[[8,2,2]]` (D14)

`LatticeComplex::new` places no lower bound on the extent, and `valid_positions` on a periodic axis
is the full extent, so a 2×2 periodic lattice has 4 vertices, 8 edges and 4 faces. The concern was
that the wrap-around at extent 2 might make a face's boundary revisit an edge or make two faces
share both edges along one axis in a way that breaks the chain complex. The probe read the boundary
matrices and the code checks off it:

| Quantity | `square_torus(2)` | `square_torus(3)` (reference) |
|---|---|---|
| cells `v, e, f` | 4, 8, 4 | 9, 18, 9 |
| Euler characteristic | 0 | 0 |
| `∂₁ ∂₂ = 0` over ℤ | true | true |
| every face boundary has 4 distinct edges, coefficients ±1 | true | true |
| every edge lies in exactly 2 faces | true | true |
| `β₀, β₁, β₂` over ℤ and over 𝔽₂ | 1, 2, 1 | 1, 2, 1 |
| `derive_code` | `[[8, 2]]`, 4 Z-checks and 4 X-checks of weight 4 | `[[18, 2]]`, 9 and 9 of weight 4 |
| homology and cohomology representative weights | 2, 2 | 3, 3 |
| `check_class_invariance` for `Z̄, S̄, T̄` on both qubits | holds, 32 states each | holds, 108 states each |
| `check_clifford_action_on_qubit` for `H̄` on both qubits | holds, 13 gates, others examined 2 | holds, 24 gates, others examined 2 |

Two faces do share both axis-0 edges, with opposite signs, which is the ordinary cell structure of a
torus built from four squares, and the complex is a torus in the usual sense. The minimum-weight
logical representative has weight 2, so the code is `[[8,2,2]]`. The composite Choi of a
`2^8 → 2^2` channel has `2^20` entries, inside the numeric path's default cap, so the fixture serves
D1's agreement scenario without the hand-built `[[4,2,2]]` fallback. The fallback stays in the
design as a fallback only.

## 2. The Frobenius-to-diamond constant is `√(d_in d_out)` (D8)

The register's candidate, `d_in · √(d_in d_out)`, is a valid bound and loose by a factor `d_in`.
The tight chain, for any linear map `Φ : L(X) → L(Y)` with the crate's unnormalised Choi operator
`J(Φ) = Σ_{ij} |i⟩⟨j| ⊗ Φ(|i⟩⟨j|)`:

1. **`‖Φ‖_⋄ ≤ ‖J(Φ)‖_1`.** Every unit vector on `X ⊗ X` is `(I ⊗ A)|Ω̃⟩` for `|Ω̃⟩ = Σ_i |i⟩|i⟩` and
   a matrix `A` with `‖A‖_F = 1`. Then `(Φ ⊗ id)(|ψ⟩⟨ψ|) = (I ⊗ A) J(Φ) (I ⊗ A)†`, and the trace norm
   is at most `‖A‖_∞² ‖J(Φ)‖_1 ≤ ‖A‖_F² ‖J(Φ)‖_1 = ‖J(Φ)‖_1`. Taking the supremum over `ψ` gives
   the bound. No Hermiticity assumption is needed.
2. **`‖J‖_1 ≤ √(rank J) · ‖J‖_F ≤ √(d_in d_out) · ‖J‖_F`**, Cauchy–Schwarz on the singular values.
3. **Lower bound.** `J(Φ)/d_in = (Φ ⊗ id)(ω)` for the maximally entangled state `ω`, so
   `‖J(Φ)‖_1 ≤ d_in ‖Φ‖_⋄`, and `‖J‖_F ≤ ‖J‖_1`.

Together, for a Frobenius residual `r = ‖J(E) − J(F)‖_F` between two channels:

```text
r / d_in  ≤  ‖E − F‖_⋄  ≤  √(d_in d_out) · r
```

The report can carry both ends. A Frobenius residual of zero certifies diamond distance zero.

**Checked numerically.** On the closed-form pair `id` against `R_z(θ)` on one qubit, whose diamond
distance is `2 sin(θ/2)`: `‖J‖_F = 2√2 sin(θ/2)` and `‖J‖_1 = 4 sin(θ/2)` at every `θ` in a sweep
over `(0, π]`, so `diamond / ‖J‖_1 = 1/2 = 1/d_in` exactly, `diamond / ‖J‖_F = 1/√2`, and the upper
bound `√(d_in d_out) · r = 2r` exceeds the diamond distance by the constant factor `2√2`. Over 300
random pairs of two-Kraus channels on `d = 2` and `d = 3`, a diamond lower bound from 61 pure
inputs including the maximally entangled state never exceeded `‖J‖_1`, and `‖J‖_1` never exceeded
`√(d_in d_out) · ‖J‖_F`; the worst observed `‖J‖_1 / ‖J‖_F` was 1.98 against the factor 2. The
requirement's θ-sweep scenario holds with the tight constant.

## 3. The Pauli-basis term cap is unnecessary for Table 1, and the reason is Haruna's Eq. (3.63) (D7)

**What the design assumed.** A Pauli fault carried through `T̄`'s program of `T`, `CS†` and `CCZ`
gates branches into a Pauli superposition bounded only by `4^w` for the representative weight `w`,
hence the cap and the qLDPC risk.

**What the algebra says.** Haruna defines every diagonal logical gate as a function of the logical
`Z̄(γᵢ)` operators and derives the physical decomposition from it by expanding modulo 2:
`S̄(γ) = exp(iπ/2 · (I − Z̄(γ))/2)` (3.14), `CZ̄(γ₁, γ₂) = exp(iπ · p₁ p₂)` (3.37),
`C^{m−1}Z̄ = exp(iπ · p₁ ⋯ p_m)` (3.49), `T̄(γ) = exp(iπ/4 · (I − Z̄(γ))/2)` (3.56), and in general
`O_k(γ₁, …, γ_m) = exp(iπ/2^{k−1} · p₁ ⋯ p_m)` with `pᵢ = (I − Z̄(γᵢ))/2` (3.63). Each such gate `G`
lies in the commutative algebra generated by the `m` Paulis `Z̄(γᵢ)`, which has dimension `2^m`.
Conjugating a Pauli fault `P = X^a Z^b` through `G` gives `G P G† = P · (P† G P) G†`, and
`P† G P` is `G` with each `Z̄(γᵢ)` replaced by `(−1)^{⟨a, γᵢ⟩} Z̄(γᵢ)`. So the remainder
`(P† G P) G†` lies in the same `2^m`-dimensional algebra: the propagated error has at most `2^m`
Pauli terms, each of the form `P · Z̄(γ_S)` for a subset `S` of the gate's logical qubits. The
representative weight `w` does not enter. For a single-block gate `O_k(γ)` and a fault with
`⟨a, γ⟩ = 1` the remainder is `exp(iπ/2^{k−1}) · O_{k−1}(γ)†`, one level down the Clifford
hierarchy; with `⟨a, γ⟩ = 0` it is the identity and nothing spreads. Z-type faults commute with
every diagonal gate and never spread.

**Checked numerically** on `w = 3, 4, 5` with the crate's polynomial `(2n³ − 3n² + 2n)/8`, which
agrees with the paper's `T ∏CS† ∏CCZ` product entry for entry: `T̄ Z₀ T̄† = Z₀`; `T̄ X₀ T̄†` has exactly
two Pauli terms, `X₀` and `X₀ Z̄(γ)`, each of modulus `1/√2`, at every `w`, that is
`X₀ · exp(±iπ/4 · Z̄(γ))`. A first attempt at the algebra predicted `2^{w−1}` terms and was wrong;
the numerics caught it, and Eq. (3.63) explains why. For contrast, `C³Z` on four qubits sends `X₀`
to `X₀ · CCZ₁₂₃`, whose remainder is a genuine non-Clifford diagonal with 8 Pauli terms, which is
still `2^m` for `m = 4` and still independent of any representative weight.

**Consequence for the Haruna filter.** The filter's answer under `pauli_weight(1)` follows from the
algebra for every CSS code and every representative weight: `Z̄` and `X̄` do not spread; every other
Table 1 gate fails a single X-type fault on its support, because the remainder is a non-trivial
logical operator (`Z̄` for `S̄`, `exp(±iπ/4 Z̄)` for `T̄`, `Z̄(γ₂)` for `CZ̄` under a fault on `γ₁`),
which no Pauli recovery of the fault's own `X` removes. That confirms the road map's oracle facts
and extends them. The implementation should still compute the verdict; the algebra is now the
provenance of the test's expected values, which the anti-circularity protocol asks for.

**Consequence for the carrier.** The propagator does not need a Pauli-basis expansion for Table 1
at all. It needs a `GaugeFieldGate` carrier: the blocks `γ₁, …, γ_m` as `Gf2Chain`s and the phase
function on `{0,1}^m` as `2^m` values of `Turns`, which is Table 1's third column and the
generalisation of `DiagonalPhase` from one block to `m`. Conjugation by a Pauli costs `m` inner
products through `Gf2Chain::inner`, the remainder is another `GaugeFieldGate` with the same blocks,
its Pauli terms are read off by a `2^m`-point transform in exact cyclotomic arithmetic over `Q(ζ₈)`
(four `Rational<i64>` coefficients, denominators `2^m`), and correctability is decided per term
against `LogicalBasis` as the spec already says. Clifford layers (`H̄`, `S̄`, `CZ̄`) go through the
tableau as before. A cap remains necessary only for a user program that interleaves two or more
non-Clifford layers with non-diagonal Cliffords, where no normal form of polynomial size is known;
the honest fallback there is a structured refusal, not a term explosion.

**Consequence for qLDPC.** The FT analysis no longer touches `w`. The emitter's tuple cap on
`logical_t` (`C(w,3)` triples) is a real cost of the physical program and stays; the fault analysis
works from the gauge-field expression and never materialises the program.

**A wording correction to this change's own text.** Table 1 has no logical `CS̄†` or `CC̄Z` rows.
`CS†` and `CCZ` are physical gates inside `T̄`'s decomposition (3.59); the logical controlled gates
are `CZ̄`, `C^{m−1}Z̄` and, in general, `O_k` with `m ≥ 2` (3.63). The register's V-3, `design.md`
D7, the proposal and the `qcl-fault-tolerance` spec name "`CS̄†`, `CC̄Z`" as Table 1 gates and should
name `T̄` and `C^{m−1}Z̄` for `m ≥ 3` instead, and the `PauliBasisToCap` label disappears with the
cap for Table 1.

## Edits these findings call for

None applied yet.

- `design.md` D14: `square_torus(2)` is the numeric fixture; the `[[4,2,2]]` complex stays as
  fallback text only. Open question 1 closes.
- `design.md` D8, register V-11, `qcl-abstraction` "Frobenius proxy" requirement and its scenario:
  the factor is `√(d_in d_out)`, with the two-sided chain above; the θ-sweep scenario gains the
  lower bound `r / d_in ≤ 2 sin(θ/2)`. Open question 2 closes.
- `design.md` D7, register V-3, proposal bullet 4, `qcl-fault-tolerance` requirements 2 and 4 and
  the filter's labels, tasks 5.2 and 5.5: replace Pauli-basis propagation with the `GaugeFieldGate`
  carrier; drop `PauliTermCountExceeded` for Table 1 and keep the cap for general programs only;
  name `T̄` and `C^{m−1}Z̄` where the text says `CS̄†`, `CC̄Z`; record the derived oracle as the
  expected values' provenance. Open question 3 closes with the cap lifted for Table 1.
