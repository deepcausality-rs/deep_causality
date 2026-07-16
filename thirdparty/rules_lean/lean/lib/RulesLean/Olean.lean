import Lean

/-!
# `RulesLean.Olean` — introspect compiled `.olean` files.

Reads the metadata embedded in compiled `.olean` files using Lean's
own `Lean.readModuleData` API. Fast (header-only reads, no body
deserialization) and faithful — same view Lean itself uses at
elaboration time.

## What's exposed

* `imports` / `importModuleNames` — modules this `.olean` imports.
* `transitiveImports` — closed under a caller-supplied resolver.
* `exportedConstants` — every declaration the module defines (its
  symbol table). Includes definitions, theorems, axioms, structures,
  inductives — anything that lands in `constNames`.
* `containsAxiom` — predicate: does the module's symbol table
  contain a constant with the given name? Cheap version of
  axiom-dependency tracking when the caller only needs to ask "does
  this module bring in axiom X?".

## What's deferred to follow-up versions

* True axiom-dependency closure (which axioms each constant
  transitively depends on across the imported module graph).
  Requires walking `ConstantInfo` records and resolving axiom
  references — bigger surface than `containsAxiom` checks.
* Content hash of the module data. No stable accessor in stock Lean
  4.29.1; would need to compute over the file bytes if exposed.
-/

namespace RulesLean.Olean

open Lean

/--
Read the `.olean` at `path` and return its declared imports.

Touches only the module header — no body decode, so fast even on
mathlib-scale libraries (~5s for 7878 oleans on Apple Silicon).
Errors propagate as plain `IO.userError`; failed reads are the
caller's problem to swallow.
-/
unsafe def imports (path : System.FilePath) : IO (Array Import) := do
  let (modData, _) ← readModuleData path
  return modData.imports

/--
The same data as `imports`, but flattened to plain module names — the
common case where callers only want the imported `Name`s, not the
full `Import` records (which also carry runtime/transitively-public
flags we rarely need).
-/
unsafe def importModuleNames (path : System.FilePath) : IO (Array Name) := do
  let imps ← imports path
  return imps.map Import.module

/--
Transitive import closure over a caller-supplied resolver.

`resolve` maps a module name to its `.olean` path; if the resolver
returns `none`, the module is treated as a leaf (typically because
it's outside the Lake workspace we're considering — e.g., Init from
the toolchain stdlib that we don't want to walk into).

Returns the set of module names reachable from `start`, including
`start` itself. Order is unspecified; uses a worklist with
deduplication.

Cycle-safe: each module is visited at most once.
-/
unsafe def transitiveImports
    (start : Name)
    (resolve : Name → IO (Option System.FilePath))
    : IO (Array Name) := do
  let mut visited : Std.HashSet Name := {}
  let mut worklist : Array Name := #[start]
  while !worklist.isEmpty do
    let some mod := worklist.back? | break
    worklist := worklist.pop
    if visited.contains mod then
      continue
    visited := visited.insert mod
    match ← resolve mod with
    | none => pure ()  -- unresolved: treat as leaf
    | some path =>
      let imps ← importModuleNames path
      for imp in imps do
        if !visited.contains imp then
          worklist := worklist.push imp
  return visited.toArray

/--
Every constant the module declares (its symbol table). Includes
definitions, theorems, axioms, structures, inductives — anything
in `ModuleData.constNames`.

Compiler-generated auxiliary declarations (`extraConstNames`) are
*not* included; those are inlined by Lean's codegen and rarely
matter at the spec/proof level.
-/
unsafe def exportedConstants (path : System.FilePath) : IO (Array Name) := do
  let (modData, _) ← readModuleData path
  return modData.constNames

/--
Does the module's symbol table contain a constant by this name?

Quick way to ask "does this olean bring in axiom X?" without
materializing the whole constants list — useful for spot-checks
("does any module import `Classical.choice`?", "which oleans
declare `sorryAx`?").

Returns `true` for any matching constant, regardless of whether
it's an axiom, theorem, definition, etc. Caller is responsible for
knowing what kind of name they're looking for.
-/
unsafe def containsConstant (path : System.FilePath) (name : Name) : IO Bool := do
  let consts ← exportedConstants path
  return consts.contains name

/--
Initialise Lean's search path against the toolchain sysroot.

Callers that invoke `imports` or `readModuleData` directly must do
this once at program start, otherwise the module deserialization
fails to resolve transitive header references. Wraps `findSysroot` +
`initSearchPath` so callers don't have to remember the incantation.
-/
unsafe def initialise : IO Unit := do
  let sysroot ← findSysroot
  initSearchPath sysroot []

end RulesLean.Olean
