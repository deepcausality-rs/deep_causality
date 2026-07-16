import RulesLean.Olean

/-!
# CLI: read `.olean` headers and print imports.

Usage:
    lake exe oleanImports <olean1> [olean2 ...]
    find ... -name '*.olean' | lake exe oleanImports

Output (one tab-separated line per import edge):
    <olean-path>\t<imported-module-name>

Thin shell over `RulesLean.Olean.imports`. The expensive work is in
the library; this file just handles arg/stdin parsing and tabular
output. Used by `lake_workspace` at materialization time to produce
the module-level import manifest that the upcoming tree-shaking
aspect (or any other consumer) reads.
-/

namespace RulesLean.Cli.OleanImports

/--
Collect file paths to scan. If `args` is non-empty, return it
verbatim. Otherwise read one path per line from stdin until EOF —
lets callers do `find ... | lake exe oleanImports` without hitting
ARG_MAX on large olean trees.
-/
partial def collectPaths (args : List String) : IO (Array String) := do
  if !args.isEmpty then
    return args.toArray
  let stdin ← IO.getStdin
  let mut paths : Array String := #[]
  let mut done := false
  while !done do
    let line ← stdin.getLine
    if line.isEmpty then
      done := true
    else
      let trimmed := (line.splitOn "\n").head!
      if !trimmed.isEmpty then
        paths := paths.push trimmed
  return paths

unsafe def run (args : List String) : IO UInt32 := do
  let paths ← collectPaths args
  if paths.isEmpty then
    IO.eprintln "usage: oleanImports <olean1> [olean2 ...]   (or pipe one path per line on stdin)"
    return 2

  RulesLean.Olean.initialise

  let mut rc : UInt32 := 0
  for path in paths do
    try
      let imports ← RulesLean.Olean.importModuleNames (System.FilePath.mk path)
      for imp in imports do
        IO.println s!"{path}\t{imp}"
    catch e =>
      IO.eprintln s!"error reading {path}: {e}"
      rc := 1
  return rc

end RulesLean.Cli.OleanImports

/-- Entry point — `lake exe oleanImports` resolves to this. -/
unsafe def main (args : List String) : IO UInt32 :=
  RulesLean.Cli.OleanImports.run args
