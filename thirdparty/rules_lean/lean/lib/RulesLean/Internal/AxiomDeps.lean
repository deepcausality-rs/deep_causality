import Lean

/-!
# `RulesLean.Internal.AxiomDeps` — axiom-dependency tools.

**Internal.** Signature and semantics may change between any two
`rules_lean` releases. The right shape for full transitive axiom-
dependency tracking isn't obvious yet — this module ships the
minimal-useful starting points and grows iteratively.

## What's exposed

* `declaredAxioms` — list the axioms a module *directly declares*
  in its symbol table. Doesn't follow references; if module A
  declares no axioms but uses `Classical.choice` from `Init`,
  `declaredAxioms` returns `#[]` for A. The transitive case is a
  separate v0.4+ project.

* `isAxiom` — predicate: does a module declare a specific axiom by
  name?

## What's NOT here yet

* True transitive axiom closure (walking each `ConstantInfo`'s type
  + value expressions, resolving every referenced `Name`, checking
  if the referenced const is itself an axiom in some imported
  module). That's the audit-grade analysis worth building, but it
  requires:
    - full module-body deserialization (slower than header-only)
    - import resolution across the workspace (recursive
      `readModuleData` calls following imports)
    - some way to cache + checkpoint progress for big libraries
  Lands as `RulesLean.Internal.AxiomDeps.transitiveClosure` once
  there's a concrete consumer pushing on the shape.
-/

namespace RulesLean.Internal.AxiomDeps

open Lean

/--
List the names of axioms a module *directly declares* in its symbol
table.

Walks `ModuleData.constants` and keeps only the `.axiomInfo` variant.
Does **not** follow references: a module that imports an axiom
without re-declaring it returns `#[]`. Useful as a one-pass scan
("which oleans introduce new axioms?") but not a substitute for a
proper transitive-axiom audit.
-/
unsafe def declaredAxioms (path : System.FilePath) : IO (Array Name) := do
  let (modData, _) ← readModuleData path
  let mut axioms : Array Name := #[]
  for c in modData.constants do
    match c with
    | .axiomInfo val => axioms := axioms.push val.name
    | _ => pure ()
  return axioms

/--
Does a module declare a specific axiom by name?

Convenience over `declaredAxioms` — cheaper if the caller only
cares about presence.
-/
unsafe def isAxiom (path : System.FilePath) (name : Name) : IO Bool := do
  let axioms ← declaredAxioms path
  return axioms.contains name

end RulesLean.Internal.AxiomDeps
