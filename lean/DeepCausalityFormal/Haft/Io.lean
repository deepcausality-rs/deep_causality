/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — IO action monad laws.

Rust source: `deep_causality_haft/src/io/` — trait `IoAction` (`Output`, `Error`, `run`) with
combinators `IoPure` (`io_pure.rs`), `IoAndThen` (`io_and_then.rs`), `IoMap`, `IoMapErr`,
`IoFail`. The module docstring states the three monad laws and says the equivalences "hold on
the value produced by `run`" — a denotational equality, which is exactly what is proved here.

Accepted theory: an `IoAction` is a nullary Kleisli arrow `() ⇝ A` over `Result` (the module's
own words); the laws are the Kleisli-triple laws (Moggi 1991) for the `Except` monad, observed
through `run`. Model: `Io E A = Unit → Except E A` (the thunk defers the effect; `run` forces
it). `IoPure::run = Ok(v)` and `IoAndThen::run = self.0.run().and_then(...)` are transcribed
one-to-one.

The docstring's three laws are the accepted three — **correct as documented**, and all three
are proved below as `run`-level equalities.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_haft/tests/algebra/formalization_law_tests.rs`.
-/

namespace DeepCausalityFormal.Haft.Io

variable {E A B C : Type}

/-- A deferred IO computation: a thunk producing a value or an error. -/
def Io (E A : Type) : Type := Unit → Except E A

/-- `IoAction::run` — force the thunk (the only effectful operation). -/
def run (m : Io E A) : Except E A := m ()

/-- `IoPure::run = Ok(value)` — the unit. -/
def ioPure (a : A) : Io E A := fun _ => .ok a

/-- `IoAndThen::run = self.0.run().and_then(|out| f(out).run())` — Kleisli bind over `Result`,
    short-circuiting on the first error. -/
def ioAndThen (m : Io E A) (f : A → Io E B) : Io E B :=
  fun _ =>
    match m () with
    | .error e => .error e
    | .ok a => f a ()

/-- Left identity on the run-denotation: `run (pure a >>= f) = run (f a)` (Rust module
    docstring law 1; Moggi 1991).

    THEOREM_MAP: `haft.io.monad_laws` -/
theorem io_left_id (a : A) (f : A → Io E B) :
    run (ioAndThen (ioPure a) f) = run (f a) := rfl

/-- Right identity on the run-denotation: `run (m >>= pure) = run m` (Rust law 2).

    THEOREM_MAP: `haft.io.monad_laws` -/
theorem io_right_id (m : Io E A) :
    run (ioAndThen m ioPure) = run m := by
  cases h : m () with
  | error e => simp [run, ioAndThen, ioPure, h]
  | ok a => simp [run, ioAndThen, ioPure, h]

/-- Associativity on the run-denotation:
    `run ((m >>= f) >>= g) = run (m >>= fun x => f x >>= g)` (Rust law 3).

    THEOREM_MAP: `haft.io.monad_laws` -/
theorem io_assoc (m : Io E A) (f : A → Io E B) (g : B → Io E C) :
    run (ioAndThen (ioAndThen m f) g) = run (ioAndThen m (fun a => ioAndThen (f a) g)) := by
  cases h : m () with
  | error e => simp [run, ioAndThen, h]
  | ok a => simp [run, ioAndThen, h]

end DeepCausalityFormal.Haft.Io
