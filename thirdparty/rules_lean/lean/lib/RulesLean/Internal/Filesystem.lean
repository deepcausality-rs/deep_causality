import Std.Data.HashSet

/-!
# `RulesLean.Internal.Filesystem` — directory + olean-tree helpers.

**Internal.** Signature, naming, and semantics may change between
any two `rules_lean` releases. Used by `RulesLean.Workspace`'s
namespace-index code; pulled out so the implementation is testable
in isolation.

## What's exposed

* `topLevelModuleNames` — list the top-level Lean module names a
  package's olean root provides. A "top-level module name" is the
  bare name (without `.olean`) of every entry at the root of the
  given directory — both `.olean` files and subdirectories
  (subdirectories denote namespaces with deeper modules inside).
-/

namespace RulesLean.Internal.Filesystem

/--
Return the top-level Lean module names a package's olean root
provides. Each name corresponds to either:

* A `.olean` file at the root (e.g. `Mathlib.olean` → `Mathlib`).
* A subdirectory at the root (e.g. `Mathlib/` → `Mathlib`).

Duplicates are deduplicated (the typical case: a package ships
both `Foo.olean` and a `Foo/` directory with submodules; both
indicate the same top-level namespace `Foo`).

Returns an empty array if the directory does not exist.
-/
def topLevelModuleNames (root : System.FilePath) : IO (Array String) := do
  unless ← System.FilePath.pathExists root do
    return #[]
  let entries ← root.readDir
  let mut seen : Std.HashSet String := {}
  for entry in entries do
    let name := entry.fileName
    -- Hidden files / build-metadata files we never want to consider.
    if name.startsWith "." then
      continue
    if name.endsWith ".olean" then
      -- "Foo.olean" -> "Foo"; "Foo.Bar.olean" -> "Foo" (first segment is
      -- the top-level namespace). splitOn dodges `String.dropRight`'s
      -- cross-version deprecation churn.
      let top := (name.splitOn ".").head!
      seen := seen.insert top
    else
      -- Subdirectory at root denotes the top-level namespace of the
      -- modules inside (skip non-directory non-olean entries).
      let path := root / name
      if (← path.isDir) then
        seen := seen.insert name
  return seen.toArray

end RulesLean.Internal.Filesystem
