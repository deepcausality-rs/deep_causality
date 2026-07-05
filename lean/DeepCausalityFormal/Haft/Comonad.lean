/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — Comonad laws.

Rust source: `deep_causality_haft/src/monad/comonad.rs` (trait `CoMonad<F>: Functor<F>`,
operations `extract`, `extend`, `duplicate`). The crate's canonical carriers are `BoxWitness`
(≅ the Identity comonad) and the tests' `IdentityWitness`. Formalized here on the **Env (reader /
coreader) comonad** `W A = E × A` — a genuinely non-trivial comonad that specializes to Identity
at `E = Unit`, so the laws are not vacuously true by carrier degeneracy.

Accepted theory: T. Uustalu & V. Vene, *Comonadic Notions of Computation*, ENTCS 203(5), 2008
(coKleisli presentation): a comonad is `(W, extract ε, extend (—)†)` with
  1. `extend extract = id`               (left identity)
  2. `extract ∘ extend f = f`            (right identity)
  3. `extend g ∘ extend f = extend (g ∘ extend f)`  (associativity)
The three informal laws in the Rust docstring are exactly these — **correct as documented**.

Rust artifact: `extract`/`extend` take `&F::Type<A>` and require `Clone`; borrowing is a
memory-management encoding with no mathematical content (the Lean model consumes values).

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_haft/tests/algebra/formalization_law_tests.rs`.
-/

namespace DeepCausalityFormal.Haft.Comonad

variable {E A B C : Type}

/-- The Env comonad carrier `W A = E × A` (environment + focus). -/
def W (E A : Type) : Type := E × A

/-- `extract`: read the focused value. -/
def extract (w : W E A) : A := w.2

/-- `extend`: rebuild the context, refocusing on the observation `f` of the whole context. -/
def extendW (f : W E A → B) (w : W E A) : W E B := (w.1, f w)

/-- Comonad left identity: `extend extract = id` (Uustalu–Vene 2008; Rust docstring law 1).
    Proof is `rfl` via product eta.

    THEOREM_MAP: `haft.comonad.laws` -/
theorem extend_extract (w : W E A) : extendW extract w = w := rfl

/-- Comonad right identity: `extract (extend f w) = f w` (Uustalu–Vene 2008; Rust law 2).

    THEOREM_MAP: `haft.comonad.laws` -/
theorem extract_extend (f : W E A → B) (w : W E A) :
    extract (extendW f w) = f w := rfl

/-- Comonad associativity: `extend g (extend f w) = extend (fun w' => g (extend f w')) w`
    (Uustalu–Vene 2008; Rust law 3).

    THEOREM_MAP: `haft.comonad.laws` -/
theorem extend_assoc (f : W E A → B) (g : W E B → C) (w : W E A) :
    extendW g (extendW f w) = extendW (fun w' => g (extendW f w')) w := rfl

/-- `duplicate = extend id` (the crate's default `duplicate`, with `Clone` standing in for
    the identity observation): each position holds the whole context. -/
theorem duplicate_eq (w : W E A) :
    extendW (fun w' => w') w = (w.1, w) := rfl

end DeepCausalityFormal.Haft.Comonad
