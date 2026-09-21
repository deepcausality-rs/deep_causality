# Test review and repair

read @AGENTS.md

Review and repair the test suite of crate: **`<CRATE>`**

Work through every file under `<CRATE>/tests/`, one file at a time. For each file: read it, diagnose
every test in it against the checklist below, repair what is broken, verify, and report. Then move
to the next file.

For a large suite (> 40 test files) you may start several agents on disjoint subdirectories, each
using these instructions.

---

## Ground rules

Non-negotiable. These come from `AGENTS.md`.

1. **Never delete a file.** Ask first. Deleting a redundant *test function* inside a file is fine
   and is often the right repair; deleting a file is not.
2. **Never git commit.** Prepare a commit message and hand it over.
3. **Never weaken an assertion to make a test pass.** If a repaired test fails, the implementation
   is wrong. Fix the implementation. If that needs refactoring with a wide blast radius, stop and
   ask.
4. **No new external crates.** Use what the workspace already ships.
5. **Register what you add.** A new test file must be declared in its `mod.rs`, the chain of
   `mod.rs` up to the root, and in `<CRATE>/BUILD.bazel`. Keep `#[cfg(test)]` correct.
6. **Do not add tests for `examples/`.** Example binaries are verified by running them.
7. **No `unwrap` in new code where an error channel exists.** Propagate with `?`, or `expect` with
   a reason when there is genuinely no channel.
8. **The suite must be green when you finish a file.** Run `cargo test -p <CRATE>` before moving on.

---

## The organizing idea

Most weak tests vary *what goes in* and then pin a single number. That misses the defect class that
matters most in numeric code: a fixture where the interesting term **vanishes**.

A test that uses a zero connection, an identity matrix, an all-zero field, two equal operands, or a
constant chosen so the answer is zero, cannot see an error in the term it is supposed to be testing.
Coverage is full, the assertion is true, and a wrong implementation passes.

So diagnose along two axes:

- **Input space** — does the test vary the input enough to expose an error?
- **Relation space** — is there a relation between *two* runs, or an identity the answer must
  satisfy, that would pin the behaviour without needing an oracle at all?

The second axis is where the cheapest and strongest repairs live.

---

## Step 1 — Read

Read the whole test file. Then read the source it tests. `tests/` mirrors `src/`, so
`tests/kernels/em/fields_tests.rs` tests `src/kernels/em/fields.rs`. Read the kernel's docstring and
body: you cannot judge whether a fixture discriminates without knowing what the code actually
computes.

Note which quantities the file's kernels produce, and which of those have a closed form, a published
value, a conservation law, or a symmetry.

---

## Step 2 — Diagnose

For every `#[test]` in the file, answer three questions.

### A. Does it observe the answer at all?

Flag any of these. Each is a defect.

| Signature | Why it is broken |
|---|---|
| `assert!(x.is_ok())` and nothing else | passes for any returned value, including a constant |
| `let _ = x.is_ok() \|\| x.is_err();` | a tautology of the excluded middle |
| `assert!(x.is_ok() \|\| x.is_err())` | same, as an assertion that cannot fail |
| `let _ = f(...);` on a semantic result | the answer is computed and discarded |
| `assert!(!v.data().is_empty())` on a fixed-size container | length is structural; cannot fail |
| `assert_eq!(t.shape(), &[..])` as the only check | shape is set by the input, not by the maths |
| `assert_eq!(x.num_dim(), 2)` as the only check | rank, not value |
| `assert_eq!(a, a.clone())` | reflexive equality; true for any derive, and for a broken one |
| `let _ = format!("{:?}", x);` | formats and discards |
| every assertion inside `if` / `if let Ok(..)` with no else | an unexpected `Err` skips the body and passes |
| `f(args)` called twice with identical arguments, results asserted equal | no deterministic pure function can fail this |
| `.get(i).copied().unwrap_or(0.0)` inside an assertion | an out-of-range read silently becomes 0.0 and satisfies the assertion |
| `assert!(x > 0.0)` where a closed form is known | excludes only zero and negatives |
| `assert!(x.is_finite())` where a value is known | excludes only NaN and infinity |
| `assert!((a - b).abs() < tol \|\| (a + b).abs() < tol)` | accepts either sign; a sign error passes |

### B. Could a plausible wrong implementation pass?

Name at least two concrete wrong implementations of the kernel — a flipped sign, a wrong constant,
a transposed index pair, a dropped term, an off-by-one in a loop bound, the wrong one of two
similar operands — and ask whether this test would notice. Flag the fixture if it would not.

Fixtures that hide defects, in the order you will meet them:

- **the interesting term is zero** — a zero connection, zero Christoffel, zero gauge potential, zero
  field, zero Riemann tensor. The term under test vanishes and nothing pins it.
- **the expected answer is zero** — a constant chosen so the result cancels. Any implementation
  scaled by a wrong factor still gives zero.
- **identity or diagonal inputs** — off-diagonal terms vanish, several distinct quantities coincide,
  and a similarity transform of the identity is the identity for any transform.
- **two operands equal** — swapping them, or using only one, passes.
- **equal magnitudes** — `|E| = |B| = 1` makes `(|E|²+|B|²)/2`, `|E|²` and `|B|²` all equal 1.
- **one size only** — an index expression can degenerate. In a 2x2 there is one off-diagonal entry
  at `j = 0`, where `a[i*n + j]` and `a[i*n - j]` are the same element.
- **only one component checked** of a multi-component result.
- **the number is in the comment and the assertion retypes the formula** — the test then asserts an
  expression was written the same way twice.
- **the oracle is derived from the code under test** — a helper in the test file that reimplements
  the kernel's own forward model, then asserts the kernel inverts it.

### C. What relation is available and unused?

For each kernel in the file, work through this catalogue and note which apply. These are the
repairs; most need no oracle.

**Symmetry and covariance (metamorphic).**
- Rotate every vector input by a rotor `R`: scalars unchanged, vectors and bivectors co-rotate.
  `f(R a R̃, R b R̃) == R f(a, b) R̃`.
- Gauge invariance: `A → A + dχ` leaves the field strength unchanged. This forces a non-zero
  connection into the fixture as a side effect, which is usually the whole point.
- Parity: vectors flip, pseudovectors do not; a pseudoscalar changes sign.
- Lorentz boost: `E² − B²` and `E·B` are invariant.
- Galilean shift: adding a constant velocity leaves the strain rate unchanged and changes the
  convective term by a known amount.
- Time reversal: `v → −v`, `B → −B`.

**Exact algebraic identities.** Free, three lines each.
- Antisymmetry and nilpotency: `a ∧ b = −b ∧ a`, `a ∧ a = 0`, `d∘d = 0`, `∂∘∂ = 0`.
- Bianchi: `∇_μ G^μν = 0`. First Bianchi on the Riemann tensor.
- Jacobi identity on structure constants.
- Decomposition: `∇u = S + Ω`, reconstructed and compared.
- Cross-kernel identities: `Pe = Re·Pr`, `Ra = Gr·Pr`, `Le = Sc/Pr`, `v_escape = √2 · v_orbital`,
  `p0/p = (ρ0/ρ)(T0/T)`. These relate kernels that share no code, so a consistently retyped formula
  cannot satisfy them.

**Conservation and monotonicity.** Energy, momentum, angular momentum, mass. Entropy does not
decrease. A decay is monotone. A distribution stays in `[0, 1]`.

**Dimensional and scale invariance.** Scale all lengths by `λ` and all times by `τ`; a quantity of
dimension `LᵃTᵇ` must scale by `λᵃτᵇ`. No oracle, and it catches a wrong exponent directly.

**Precision as a differential oracle.** Every kernel here is generic in its scalar. Run the same
computation at `f64` and at `deep_causality_num::Float106`; the `f64` answer must agree with the
`Float106` one to about `f64` precision. This measures conditioning instead of guessing at it, and
needs no external truth. Reach for it wherever the kernel subtracts nearly-equal numbers.

**Convergence order.** For anything discretized, halve the step and check the error falls by `2^p`
for the claimed order `p`. A wrong coefficient shows up as a wrong order even when a single value
looks plausible.

**Manufactured solutions.** Pick a solution, substitute it into the operator, derive the source term
it requires, and check the solver recovers it. The right tool for PDE and field-equation kernels.

**Round trip.** Any kernel with an inverse: `f⁻¹(f(x)) == x` across the domain and both branches.

**Differential against a second path.** Where the same quantity has two formulations, compute both
and compare.

**Published values.** A cited table or textbook number, with the citation in the test. Prefer this
over a retyped formula whenever one exists.

---

## Step 3 — Repair

Priority order. Do not churn; a test that already discriminates is done, leave it alone.

1. **Tests that cannot fail** (every signature in table A). Each needs one real assertion. If the
   test is superseded by another that does the same job properly, delete the test function and say
   so in the report.
2. **Tests whose name promises what the body does not do.** Either make the body test the name, or
   rename it to what it actually does. A test named for an error path that asserts `is_ok()` is a
   lie in the suite.
3. **Fixtures where the interesting term vanishes.** Replace the zero connection, identity matrix or
   equal operands with something that discriminates, and assert the relation from part C. This is
   where the real bugs are.
4. **Bare `is_err()`.** Pin the variant and, where the message carries information, a substring of
   it: `match err.0 { PhysicsErrorEnum::X(msg) => assert!(msg.contains("...")), e => panic!(...) }`.
   A refusal for the wrong reason must fail.
5. **The number is in the comment.** Keep the formula assertion and add the independent number as a
   second assertion. Both, so the second breaks the circularity.
6. **Single point where a range is meaningful.** Convert to a table spanning the physical range,
   log-spaced where the quantity is scale-free, including the extremes the kernel claims to support.
7. **Corner cases the physics has**: zero, one, the boundary of validity and one ulp either side of
   it, smallest and largest representable, and every documented discontinuity. Prefer `M = 1 ± ε`
   over `M = 0.5` against `M = 25`.
8. **Tolerances.** Replace a magic absolute tolerance with a relative one, or with a stated number of
   ulp, so the test survives a change of `FloatType`. Say in a comment what the tolerance measures.

While repairing:

- Delete the reasoning you did not mean to commit. No `// Wait`, `// Actually`, `// Let's try`,
  `// I implemented`, `// Skip this test`, `// TODO`. Document what the code does, not how you got
  there.
- Split a test that asserts six unrelated behaviours into six tests. A failure in the second must
  not hide the other four.
- Remove `unwrap_or` defaults from assertions. Index directly so a malformed result panics.
- Keep the file's existing conventions: its tolerance constants, helper names, section headers.

---

## Step 4 — Verify

In order:

1. `cargo test -p <CRATE>` — green, and the count went up or stayed level for the right reason.
2. **Sabotage check.** For at least one repaired kernel per file, replace its body with a constant
   (`Ok(R::zero())`, or the identity) and confirm the repaired test now fails. Restore the source
   immediately and confirm `git status` is clean apart from your test edits. A repair that survives
   this is real; one that does not is decoration.
3. **Targeted mutation testing** once a family is done:
   `cargo mutants -p <CRATE> --file '<CRATE>/src/<path>/<file>.rs' -j 8`.
   A survivor is a decision nothing pins. Either add the test that kills it, or, if the mutation
   provably cannot change behaviour, add it to `.cargo/mutants.toml` with the measurement that
   settles it and confirm the entry matched using the `comm` check that file documents.
4. `make format && make fix` once the crate is done.

---

## Step 5 — Report per file

Keep it short and factual:

- File, number of tests, how many were already sound.
- Each defect found: test name, which signature from A or B, and what a wrong implementation would
  have got away with.
- Each repair: what relation or oracle you added.
- Sabotage check: which kernel, which test caught it.
- Mutants: run or not, survivors listed.
- Anything you left alone, and why.

Do not report a test as repaired unless the suite is green and the sabotage check passed.

---

## Stop and ask

Stop, report, and wait when:

- A repaired test fails and the fix is in the implementation with a wide blast radius.
- A kernel's documented behaviour and its actual behaviour disagree, and which one is right is a
  product decision.
- The file documents a known defect rather than fixing it ("this test documents the current
  behavior"). Say what the defect is and ask whether to fix the code.
- A fixture cannot be made to discriminate without an oracle the crate does not have.
- You would need to delete a file.

Do not fabricate certainty to keep moving. A test you cannot make discriminate is a finding, not a
failure.

---

## Progress

Keep a running list of files done, in progress, and not started, so the work is resumable across
invocations. Report it at the end of every file.

---

## Commit message

At each natural group boundary, once `cargo test -p <CRATE>` and clippy are green, prepare a commit
message in the house style and ask for it to be committed. Do not commit.

```
test(<CRATE>): <what the group changed>

<what was wrong, in terms of what a wrong implementation could have got away with>
<what relation or oracle now pins it>
<sabotage check and mutant results>

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>
```
