import RulesLean.Internal.Closure

/-!
# CLI: compute the precise olean closure for one or more `.lean` sources.

Usage:
    lake exe leanClosure <manifest.tsv> <src1.lean> [src2.lean ...]

Output: one absolute olean path per line — the minimal set of
oleans the listed source(s) transitively need on LEAN_PATH. Modules
outside the workspace (stdlib `Init`, `Std`, `Lean.*`) are treated
as leaves and don't appear in the output (they're already on the
toolchain LEAN_PATH).

Built on `RulesLean.Internal.Closure` — same library powers any
future tree-shaking aspect or analytics tool.
-/

namespace RulesLean.Cli.LeanClosure

open Lean

def run (args : List String) : IO UInt32 := do
  let (manifestPath, sources) := match args with
    | manifest :: rest => (manifest, rest)
    | [] => ("", [])
  if manifestPath.isEmpty || sources.isEmpty then
    IO.eprintln "usage: leanClosure <manifest.tsv> <src1.lean> [src2.lean ...]"
    return 2

  let idx ← RulesLean.Internal.Closure.loadIndex (System.FilePath.mk manifestPath)

  -- Collect direct imports from every source.
  let mut seeds : Array Name := #[]
  for src in sources do
    let imps ← RulesLean.Internal.Closure.extractImports (System.FilePath.mk src)
    seeds := seeds ++ imps

  -- Transitively close.
  let allModules := RulesLean.Internal.Closure.closure idx seeds

  -- Resolve each module name to its olean path. Print only modules
  -- present in the workspace's manifest (stdlib not included).
  for mod in allModules do
    match idx.modulePath.get? mod with
    | some path => IO.println path.toString
    | none => pure ()

  return 0

end RulesLean.Cli.LeanClosure

/-- Entry point — `lake exe leanClosure`. -/
def main (args : List String) : IO UInt32 :=
  RulesLean.Cli.LeanClosure.run args
