# Note: the builder hides the categories — the causal process builder

Status: **strategic / partially specified.** Captured from a working session on
`causal-arrow-strength`. Confirms and extends §8 of
`causal-arrow-generalization.md` ("the builder *is* the syntax of the Arrow").
The principle here is a binding design direction; the `PropagatingEffect`/
`PropagatingProcess` instantiation is sketched, not fully specified.

## 0. The one-sentence principle

> **Builders are the user-facing syntax; the categorical algebra — witness
> `Morphism`, value-level `Arrow`, the causal monad, the HKT witness pattern — is
> the *desugared form* the user never names.** A fluent chain is the textual form of
> a string/wiring diagram; the types thread through `Self` but stay camouflaged.

This is the ergonomic counterpart to the whole program: the algebra makes the
generalization *provable*; the builder makes it *usable*. Neither replaces the other.

## 1. What already exists (the proof the pattern works)

The **CDL** (causal discovery DSL) already hides the witness pattern and monadic
composition behind a typestate builder: each fluent method advances a witness in
`Self`'s type and applies the monadic op underneath. The user writes
`load → clean → preprocess → discover → analyze → finalize` and never sees the
witness/GAT machinery or the `bind`s. That is the existence proof that "categorical
complexity behind a builder" is real, shippable Rust — not aspiration.

## 2. The extension (the new idea)

Extend the same hiding to the rest of the algebra:

- **Arrow layer (`causal-arrow-strength`, now).** The value-level `Arrow`
  combinators (`Compose`/`First`/`Second`/`Split`/`Fanout`) are the desugared
  algebra. Their **fluent methods are the builder syntax** (`.compose`/`.then`,
  `.split`/`.par`, `.fanout`). The user writes a left-to-right chain; the deeply
  nested combinator types (`Compose<Split<…>, …>`) are never named. `#[diagnostic::
  on_unimplemented]` + sealed traits keep mis-typed chains legible. This is the
  resolution of the "two arrow notions" concern: the user sees neither the witness
  `Morphism` nor the value `Arrow` structs — only the builder.

- **Effect / process layer (`causal-arrow-cdl-unification`, later — needs the
  carrier).** A **causal process builder** that hides the monad + witness over the
  `PropagatingEffect` carrier (§10) and `PropagatingProcess` (the State/Writer
  enrichment, §10 `lift`). Shape (illustrative, not final):

  ```text
  process(evidence)                 // lift raw data into a PropagatingEffect
      .then(discover)               // discover : Effect -> Effect   (static arrow inside)
      .then(generate_model)         // §11 pure functor, Effect -> Effect
      .infer(model ⊗ evidence)      // Kleisli fragment = the causal monad's bind
      .govern(teloi)                // endomorphism on Effect
      .act(state)                   // Mealy step
      .run()                        // -> final PropagatingEffect (the monad's value)
  ```

  Every `.then`/`.infer` desugars to the appropriate fragment (static `Arrow`
  composition vs. Kleisli `bind`) over the **shared object `PropagatingEffect`**.
  The builder is what makes "the last effect exiting the CDL is already the monad's
  value" (§10) ergonomic instead of ceremony.

## 3. Why this belongs with the generalization, not after it

§8 already argues the builder is not *hiding* the category theory — it **is** the
category theory's term syntax (a well-typed wiring diagram). So the builder is part
of the *artifact/mechanization* evidence for Paper 2, not a separate cosmetic layer:
"seven-to-twelve fluent lines do discover → infer → act with the CT machinery
shielded" is the adoption proof that the algebra is real code. Specifying the builder
alongside the Arrow keeps the algebra and its syntax co-designed.

## 4. The invariant the builder must not break (§10)

The builder may hide the *machinery* but must not blur the *separation*: **data flows
as `PropagatingEffect`; static structure (graph, SURD lattice, manifold metric) stays
a parameter captured by a stage, never payload in the flowing effect.** A `.with_graph(g)`
/ `.parameterize(...)` step captures structure as a parameter; a `.then(...)` threads
the effect. Blur this — push the graph into the effect and `bind` it — and discovery
collapses to Kleisli and "Arrow ⊋ monad" evaporates. The builder's method set should
make the parameter-vs-payload distinction syntactically obvious.

## 5. Scope split (what is specified where)

- **`causal-arrow-strength`:** the generic **arrow builder** — the fluent surface
  over the value-level `Arrow` that hides the combinator types. Buildable now in
  `deep_causality_haft`; no `PropagatingEffect` dependency.
- **`causal-arrow-cdl-unification`:** the **causal process builder** over
  `PropagatingEffect`/`PropagatingProcess` — instantiates the same builder pattern
  on the shared carrier, with the `infer` (Kleisli) and `govern`/`act` fragments.
  Depends on the §10 carrier rework; full spec deferred to that change.

## 6. Open questions

- Method vocabulary: `.then`/`.par`/`.fanout`/`.with_*` vs. the categorical names
  (`.compose`/`.split`/`.fanout`). Lean: expose *both* — categorical names on the
  `Arrow` trait, friendlier aliases on the builder.
- How much the builder can erase the combinator-struct types from error messages
  without `dyn` (sealing + `on_unimplemented` only soften, never fully remove).
- Where the builder terminal lives: `.run(input)` (apply) vs. `.build()` (yield the
  composed `Arrow` value for reuse).
