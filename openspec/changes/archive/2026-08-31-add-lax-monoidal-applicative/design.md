## Context

`deep_causality_haft` presents `Applicative<F>: Functor<F> + Pure<F>`, following Haskell's hierarchy. `Pure::pure` takes one value by value, bounded only by `Satisfies<C: ?Sized>`, an empty marker with no supertrait. A fixed-arity product of arity ≥ 2 cannot be built from one moved value, so `Complex`, `Quaternion`, `Octonion` and `Dual` are excluded from `Applicative` and, because `Monad` also names `Pure`, from `Monad` as well. This is `unified_math_gaps.md` §4.1 items E1 and E2.

The investigation behind this change is `openspec/notes/hkt_gat/monoidal-applicative.md`, which surveyed 57 witnesses across the workspace and put each finding to an independent agent instructed to refute it. Every compiler claim in this document was run.

## Goals / Non-Goals

**Goals:**

- Admit fixed-arity products into an applicative hierarchy without requiring `Clone` of anything that does not already have it.
- Close E1 for `Complex`, `Quaternion`, `Octonion` and E2's functor layer for `Dual`.
- Make the choice of monoid a witness claims visible to the type system rather than buried in a method body.
- Keep the change additive, with no coordinated cutover and no behaviour change to any existing caller.

**Non-Goals:**

- Changing `Applicative`, `Pure`, `Monad`, `Traversable`, `Arrow` or the effect system.
- Adopting the new stack on the shape-carrying witnesses. All 12 topology witnesses, plus `CsrMatrix`, `DenseMatrix`, `DenseVector`, `BTreeMap`, `HashMap` and `Cofree`, have a φ that is total only on a shared context or matching shape and want an error channel the plain signature does not have. Deferred.
- Adopting it on `Vec`, `LinkedList` or `VecDeque`, which have no lawful diagonal-free instance.
- Touching `CausalTensor`, `CausalTensorTrain`, `CausalMultiField`, `GaugeField`, `LatticeGaugeField`, `CurvatureTensor`, `DeRhamSharpIso`, `Free`, `Tuple2/3` or `ResultUnbound`, which have no lawful φ at all.
- Auditing the `transmute_copy` layer under `CausalMultiFieldWitness`. It wants a separate look and this change must not make it worse.

## Decisions

### `zip_with` is the primitive; `zip` is derived

**Alternative considered:** `zip` as the primitive, the textbook presentation of φ.

Rejected on measurement. Deriving `apply` from `zip` builds `F::Type<(Func, A)>` and hands it to `fmap`, so the tuple must satisfy the witness constraint:

```
error[E0277]: the trait bound `(Func, A): Satisfies<<F as HKT>::Constraint>` is not satisfied
note: required by a bound in `fmap`
```

Adding that bound to `apply` fixes the trait and then leaks into every function generic over the witness. `zip_with` never constructs the tuple, so the bound disappears from `apply` and from callers, and `zip` carries it only where tuples are genuinely wanted. All 86 real witnesses use `NoConstraint`, which has a blanket impl, so the obligation is latent rather than active today; it would still have to be written into every generic signature.

The shape has precedent in the crate twice: `MonoidalMerge::merge` at the `HKT3Unbound` level, and `LatticeGaugeFieldWitness::zip_with` concretely in the topology crate.

### `Semigroupal` and `LaxMonoidal` are split

**Alternative considered:** one trait carrying both `unit` and `zip_with`.

Rejected because every context-carrying witness in the workspace has a lawful φ and no lawful η. `pure(a)` at least receives a value; `unit()` receives nothing and must still name a complex, a grade, a lattice or an adjacency map. Bundling them excludes the entire topology crate from a structure it can otherwise support, and pushes those witnesses toward writing an unlawful `unit`. That failure has already occurred once here: the deleted `GaugeField` `MonoidalMerge` impl, where five tests asserted the defect as the specification.

`apply` needs φ alone. η is required only to state the unit laws.

### The derived `apply` goes on a new trait rather than into `Applicative`

**Alternative considered:** change `Applicative`'s supertraits from `Pure<F>` to `Convolutional<F>` and make `apply` a provided method.

Rejected, and this is the decision the whole change turns on. `Vec` has no lawful diagonal-free instance. `ZipList` has no unit, because the unit of a positional zip is the infinite repeat and a finite `Vec` cannot represent it. The cartesian product is pinned as the only lawful applicative by `haft.monad.applicative_coherence`, and it needs `Func: Clone`, which the diagonal-free signature forbids and which an impl may not re-add. The menu is empty rather than long. Two measured consequences on `VecWitness`:

```
non-Clone FnMut Func accepted today : [11, 21]        // closure captured &mut log
stateful FnMut, today               : [11, 22, 33]    // one f reused across args
stateful FnMut, cloned per pair     : [11, 12, 13]    // f reset per pairing
```

The first is a hard compile break; a closure capturing by `&mut` is `FnMut` and not `Clone`, and today's `apply` accepts it. The second is a silent behaviour change on code that keeps compiling. Neither is excused by the "laws are stated for pure functions" caveat, which excuses a law failure, not a change in the result of a legal call.

Putting the derivation on `MonoidalApplicative` makes everything else additive. It also avoids a second problem: changing `Applicative`'s supertraits would drop `Pure` from scope for `Traversable`, whose `sequence` calls `M::pure(None)` and gets it today only through `Applicative: Pure`.

### The markers are traits, not documentation

**Alternative considered:** reuse `deep_causality_algebra::Associative<O: Operator>` across both levels.

Rejected. `Associative` is a marker on *types*; this is a law on *endofunctors*. That trait's own documentation insists every impl is one deliberate promise about one type, and reusing it across levels would weaken exactly what makes it trustworthy. Siblings at the functor level are the clean move.

The markers earn their place because there are two routes to an applicative and they differ exactly on the diagonal. The monoidal route pairs slot with slot and consumes nothing twice. The monadic route induces `ap(ff, fa) = bind(ff, |f| fmap(fa, f))`, which re-runs the continuation once per function. A witness on both routes owes the coherence law that they agree, which `Monad.lean` already proves and which its docstring currently only recommends stating per witness in prose.

### `LaxMonoidal`, not `Monoidal`, in its own module

`haft/src/monoidal/` holds `SymMonoidal`, the value-level cartesian PROP. Its `unit<M: Monoid>() -> M::empty()` is a different η at a different level, and two `W::unit()` calls reachable from one crate root meaning different things is a trap. The name also reads backwards: `SymMonoidal` is the structure that *has* a diagonal, and the new trait exists precisely to be the one that does not.

A new `src/lax_monoidal/` module with doc-links in both directions keeps the 30 lines of cartesian-PROP prose in `src/monoidal/mod.rs` describing one structure rather than two. The same reasoning applies to the `haft.monoidal.*` theorem prefix, which is already occupied.

### Dropping `Dual`'s struct bound

The bound kills the witness at the GAT, before any method exists to argue about: 268 errors, every one E0277 and every one attributed to `required by a bound in Dual`. It also makes `Dual<()>` and `Dual<(A, B)>` ill-formed, so `unit` and `zip` have no well-formed return type, `Real`'s domain being neither closed under × nor possessed of the unit object.

The objection worth taking seriously is that `Dual`'s design rests on nesting for higher derivatives. It does not rest on the struct bound. Nesting rests on `impl<T: Real + Div<Output = T>> Real for Dual<T>`, which keeps its own bound, and the second and third derivatives compute correctly with the struct bound removed. `cargo check --workspace --all-targets` reports 0 errors. This is the call already made for `Complex<T>` and `Octonion<F>`.

### `CoMonad` for `Dual` is deferred

**Decided.** E2 as written in `unified_math_gaps.md` asks for `Functor` and `CoMonad` on `Dual`. This change ships the functor layer and defers the comonad.

Two reasons, and the second settles it.

There is no forced answer. `Dual<A> ≅ A^S` over the two-element index set `S = {re, du}`, and lawful comonads on `A^S` whose `extract` evaluates at a fixed identity correspond to monoid structures on `S`. A two-element set with a chosen identity carries exactly two, so exactly two comultiplications are lawful, both measured against all four laws:

```
carrier D{re,du}, extract = re, w = D{re:1, du:2}

swap       counit=true   right_id=true   assoc=true   coassoc=true
absorbing  counit=true   right_id=true   assoc=true   coassoc=true
same       counit=false  right_id=true   assoc=true   coassoc=true
             counit gave D { re: 1, du: 1 }, wanted D { re: 1, du: 2 }
```

The third shape, handing both slots the same focus, is unlawful rather than merely degenerate; it fails the counit law. An earlier draft of this design listed it as a legitimate alternative, which was wrong.

And there is no caller. On a tensor or a manifold the comultiplication buys stencils and local field operations, which is why `CausalTensorWitness::extend` rotates through `shifted_view` and `ManifoldWitness::extend` walks a cursor over the complex. On a two-slot product `extend` computes something per channel that can see both channels, and nothing in the workspace wants that. Picking one of two lawful-but-arbitrary structures to satisfy a gap item, with no consumer to validate the choice against, is how a wrong default gets locked in and then depended upon.

The `Functor` and `Foldable` halves are unaffected. Structural traversal and precision migration are real uses, and they close the part of E2 that has a reason to exist.

**If the deferral is ever lifted**, prefer **absorbing** over **swap**. Swap makes `extend` observe a dual whose derivative channel holds a function value, which carries no meaning in forward-mode AD terms.

**Alternative considered:** ship `CoMonad` with `absorbing` and document the choice. Rejected because a documented arbitrary choice is still an arbitrary choice, and the public API would then have to keep it.

### `Traversable` for `VecWitness` is withdrawn

**Decided, after implementing it.** `Semigroupal::zip_with` does express a `sequence` that
`Applicative::apply` cannot: the `apply` fold has to lift a partially-applied `push` into `M`, so it
requires the anonymous closure type to satisfy `M::Constraint`, which `sequence` cannot declare and
an impl may not add. The `zip_with` fold keeps the combining function outside `M`, compiles, and
passes. That finding stands and is why `zip_with` is the primitive.

Shipping it does not. The impl is only reachable if `Traversable::sequence`'s inner bound moves from
`M: Applicative<M>` to `M: Semigroupal<M> + Pure<M>`, and those are **substitutive, not
comparable** — neither implies the other, so this swaps one admissible population for another rather
than widening. Measured across the workspace, admissible inner witnesses go from 19 to 3:

```
LOST eligibility as inner applicative (16)
  BoxWitness                       CausalTensorWitness       ManifoldWitness
  CausalMultiFieldWitness          CsrMatrixWitness          MyEffectHktWitness{,4,5}
  CausalMultiVectorWitness         DenseMatrixWitness        VecWitness
  CdlEffectWitness                 DenseVectorWitness        LinkedListWitness
  GraphGeneratableEffectWitness    StudyEffectWitness
```

Four are the effect monads, which are what a downstream caller would most plausibly sequence over;
`Option<StudyEffect<A>> → StudyEffect<Option<A>>` would stop compiling. That nothing calls `sequence`
today makes the regression invisible, not absent. One carrier gained does not pay for sixteen lost.

The two existing impls are compatible either way — their bodies are byte-identical under both
bounds, since neither ever called `M::apply`. Impl compatibility was the wrong thing to check; the
cost falls entirely on callers.

**Alternative considered:** `M: Applicative<M> + Semigroupal<M>`, to be additive. Strictly worse: it
is more restrictive than either bound alone, losing the same sixteen plus any `Semigroupal`-only
witness.

**Precondition for revisiting.** Adopt `Semigroupal` across the effect witnesses, `Box` and
`LinkedList` first; then the bound can move without narrowing the contract, and `VecWitness` can have
`Traversable` for free.

## Risks / Trade-offs

**Two applicative traits in the vocabulary** → A witness holding both owes a law test that they agree, carried under `haft.lax_monoidal.apply_agreement`. This is the price of not breaking class C, and it buys an additive migration.

**Rewriting law tests against coherence would lose information** → The monoid coherence conditions do not pin `Vec`'s applicative: both the function-major and argument-major cartesian products satisfy all of them. What selects the one `Vec` implements is `haft.monad.applicative_coherence`, which lives outside the monoidal structure. Keep that law stated alongside rather than replacing it.

**A witness could adopt `MonoidalApplicative` while its hand-written `apply` broadcasts** → `apply` can broadcast one `Func` over n arguments without `Clone`, since one owned `FnMut` is called n times; `zip_with` cannot, because n pairings need n owned functions. For a broadcasting witness the derived `apply` is a different function. `ManifoldWitness`, `CausalTensorWitness` and `CsrMatrixWitness` are explicitly out of scope, and the spec requires a documented decision before any broadcasting witness adopts.

**`Dual`'s widened storage admits nonsense values** → `Dual<String>` becomes constructible as data. It carries no arithmetic, because every operation keeps its own `T: Real` bound. This is the same trade already accepted for `Complex` and `Octonion`.

**The `CausalMultiField` HKT layer is `transmute_copy` under a contract that a tuple payload violates** → Out of scope here, and the spec forbids giving it a `zip`. Today's `apply` panics unconditionally, which is loud and safe; a derived `apply` would trade the panic for undefined behaviour.

## Migration Plan

Additive throughout, so there is no cutover and no rollback coordination.

1. Land the four haft traits. Nothing implements them yet; the crate compiles unchanged.
2. Land the Lean file and its Rust witnesses. CI's `theorem-map` job passes once both bridge sides exist.
3. Adopt per witness, one at a time, each behind its own law tests: Cayley-Dickson family, then `Dual` after its struct bound comes off.
4. **Withdrawn.** `Traversable` for `VecWitness` was implemented, measured and reverted; see the
   decision above. It is reachable only by moving `sequence`'s inner bound from `Applicative` to
   `Semigroupal + Pure`, which costs sixteen admissible inner witnesses to gain one. Revisit only
   after `Semigroupal` is adopted across the effect witnesses, `Box` and `LinkedList`, so the bound
   can move without narrowing the trait's contract.
5. Update the website formalization page's row count last, once the theorem ids are final.

Rollback at any step is the removal of an unused trait or an unused impl.

## Open Questions

- Whether the deferred class B witnesses want `zip_with` returning `Result<F::Type<C>, Self::Error>` or an associated-error variant of the trait. The topology crate has already chosen `Result` for its own binary operations, which is evidence but not a decision.
- Whether `MonoidalMerge` should eventually be re-founded on `Semigroupal`. They are the same structure map at different kinds, and a blanket bridge is not expressible over an arbitrary `P: HKT3Unbound`, so this would be a per-carrier bridge and there is only one production carrier.
