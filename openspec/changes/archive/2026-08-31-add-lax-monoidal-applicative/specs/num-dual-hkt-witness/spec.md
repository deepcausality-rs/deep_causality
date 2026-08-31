## ADDED Requirements

### Requirement: `Dual` carries no struct-level bound
`deep_causality_num_dual` SHALL declare `pub struct Dual<T>` without a struct-level bound, matching `Complex`, `Quaternion`, `CausalTensor` and `CausalMultiVector`. Every arithmetic, analytic and algebra-tower impl SHALL keep its own `T: Real` bound, so what may be computed is unchanged and only what may be stored widens. The rustdoc SHALL state the storage-versus-computation split.

#### Scenario: The workspace is unaffected

- **WHEN** the bound is removed and `cargo check --workspace --all-targets` is run
- **THEN** it reports zero errors, because no call site relied on the bound being implied by the struct

#### Scenario: Nested differentiation still works

- **WHEN** `f(x) = x³ + 2x` is evaluated at 3 through `Dual<Dual<f64>>` and `g(x) = x⁴` at 2 through `Dual<Dual<Dual<f64>>>`
- **THEN** the results are 33, 29, 18 and 16, 32, 48 respectively, since nesting rests on `impl<T: Real + Div<Output = T>> Real for Dual<T>`, which keeps its own bound

#### Scenario: The GAT becomes well formed

- **WHEN** `impl HKT for DualWitness { type Type<T> = Dual<T>; }` is written
- **THEN** it compiles, where under the struct bound it failed with 268 errors, every one E0277 and every one attributed to the bound on `Dual`

### Requirement: `DualWitness` provides the structural functor layer
`deep_causality_num_dual` SHALL add `DualWitness` implementing `HKT` with `NoConstraint`, plus `Functor` and `Foldable`. `fmap` SHALL map `re` and `du` independently. This narrows `unified_math_gaps.md` §4.1 item E2, which also asked for `CoMonad`; see the deferral requirement below.

#### Scenario: Unrelated payload types map

- **WHEN** `fmap` is applied to a `Dual<f64>` with a function into an unrelated type such as `String`
- **THEN** the result is a `Dual<String>` with both slots mapped

#### Scenario: A non-numeric payload is admissible

- **WHEN** a `Dual` is constructed over a payload that is neither `Real` nor `Clone`
- **THEN** it is well formed and `zip_with` over it compiles

### Requirement: The functor is documented as structural, not as automatic differentiation
The `DualWitness` documentation SHALL state that this `fmap` maps `re` and `du` independently, which is the pair functor and carries no chain rule, and that it serves structural traversal and precision migration rather than forward-mode AD. It SHALL point to the arithmetic impls as the place where the chain rule lives.

#### Scenario: The caveat is present where it can be seen

- **WHEN** a reader reaches `DualWitness` expecting differentiation
- **THEN** the module documentation states plainly that `fmap` is not differentiating

### Requirement: `CoMonad` for `Dual` is deferred, and the reason is recorded
`DualWitness` SHALL NOT implement `CoMonad` in this change. The `CoMonad` half of `unified_math_gaps.md` §4.1 item E2 is deferred on the grounds that it has no identified caller and no forced answer, while the `Functor` and `Foldable` halves have clear uses in structural traversal and precision migration. The rationale SHALL be recorded in the crate documentation so the absence reads as a decision rather than an oversight, and a later change that adds it SHALL restate the choice below rather than rediscover it.

The comultiplication has no canonical form. `Dual<A> ≅ A^S` for the two-element index set `S = {re, du}`, lawful comonads on `A^S` whose `extract` evaluates at a fixed identity correspond to monoid structures on `S`, and a two-element set with a chosen identity carries exactly two. Both were measured to satisfy the counit, right-identity, extend-associativity and duplicate-coassociativity laws:

- **swap**, from the group ℤ/2 where `du · du = re`: `duplicate(w) = Dual { re: w, du: Dual { re: w.du, du: w.re } }`
- **absorbing**, from the idempotent monoid where `du · du = du`: `duplicate(w) = Dual { re: w, du: Dual { re: w.du, du: w.du } }`

A third shape that looks natural, `duplicate(w) = Dual { re: w, du: w }`, is unlawful: it fails the counit law, returning `Dual { re: w.re, du: w.re }` where `extend(w, extract)` must return `w`. Should the deferral ever be lifted, **absorbing** is the better default, because **swap** makes `extend` observe a dual whose derivative channel holds a function value, which carries no meaning in forward-mode AD terms.

#### Scenario: The absence is documented rather than silent

- **WHEN** a reader looks for `CoMonad` on `DualWitness`
- **THEN** the module documentation states that it is deferred, that no caller wants it, and that two lawful comultiplications exist with no principle selecting between them

#### Scenario: Nothing depends on a comonadic `Dual`

- **WHEN** the workspace is built without `CoMonad` on `DualWitness`
- **THEN** it compiles, confirming the deferral costs no existing call site

### Requirement: `Dual` joins the monoidal applicative stack
`DualWitness` SHALL implement `Semigroupal`, `LaxMonoidal`, `Convolutional` and `MonoidalApplicative` on the same componentwise basis as the Cayley-Dickson family, since `Dual` is a two-field product with no shape. `zip_with` SHALL be total and SHALL require neither `Clone` nor arithmetic.

#### Scenario: The unit is the all-unit dual

- **WHEN** `unit()` is called
- **THEN** it returns `Dual { re: (), du: () }`, the sole inhabitant of `Dual<()>`

#### Scenario: The crossed pairing is rejected

- **WHEN** the swap variant pairing `re` with `du` across the two inputs is tested against the left unit law
- **THEN** the law fails, confirming componentwise is the lawful choice
