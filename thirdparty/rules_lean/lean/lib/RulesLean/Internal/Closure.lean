import Lean

/-!
# `RulesLean.Internal.Closure` — transitive olean closure from a manifest.

**Internal.** API shape will evolve as the tree-shaking work lands a
concrete consumer (a Bazel aspect, an analytics CLI, etc.) and
pushes back on these signatures.

## What's exposed

* `ManifestIndex` — parsed `lake_imports_manifest.tsv` indexed two
  ways: `module-name → olean-path` and `module-name → imports`.
* `loadIndex` — parse the manifest TSV from disk.
* `closure` — transitive import closure starting from a set of
  module names.
* `extractImports` — regex-extract `import X.Y.Z` declarations from
  a `.lean` source. No Lean elaboration, just textual prefix
  scanning. Handles `--` line comments and stops at the first
  non-import top-level statement.

## How module names are derived from olean paths

Manifest lines look like:

    lake_ws/.lake/packages/mathlib/.lake/build/lib/lean/Mathlib/Data/Finset/Basic.olean    Mathlib.Order.Defs

The path's segment after `.../lib/lean/` strips down to
`Mathlib/Data/Finset/Basic.olean`. We drop the `.olean` extension
and replace `/` with `.` to recover `Mathlib.Data.Finset.Basic`.
Edge cases (path doesn't contain `/lib/lean/`, no `.olean` suffix)
fall through silently — those entries can't be looked up but don't
crash the indexer.
-/

namespace RulesLean.Internal.Closure

open Lean

/--
Indexed view of a `lake_imports_manifest.tsv`. Two maps share the
same key (module name):

* `modulePath` — points to the olean file that declares the module
  (for staging into a precise LEAN_PATH).
* `imports` — the modules this module imports directly (for
  transitive closure walks).

A module appears in `imports` even if it has no imports (empty
array) so callers can distinguish "module not in this workspace"
(`getD #[]`) from "module exists with no imports".
-/
structure ManifestIndex where
  modulePath : Std.HashMap Name System.FilePath
  imports : Std.HashMap Name (Array Name)
  deriving Inhabited

/--
Convert an olean filesystem path into the Lean module name it
declares. Returns `none` if the path doesn't fit the Lake 5+
`.lake/build/lib/lean/<Module/Path>.olean` shape we know about.
-/
def moduleNameOfPath (path : String) : Option Name :=
  -- Find the `/lib/lean/` marker; the segment after it is the
  -- module-relative path.
  match path.splitOn "/lib/lean/" with
  | _ :: rel :: _ =>
    -- Drop the .olean suffix.
    match rel.splitOn ".olean" with
    | modulePath :: "" :: _ =>
      -- Replace `/` with `.` to form the Lean name. Note: this
      -- builds a single-segment `Name` even for nested paths like
      -- "Foo.Bar"; works for string lookup but isn't structurally
      -- equal to `Lean.Name.mkStr (Lean.Name.mkStr .anonymous "Foo") "Bar"`.
      -- TODO(v0.4): structural Name construction for nested namespaces.
      let segments := modulePath.splitOn "/"
      let nameStr := ".".intercalate segments
      some (Name.mkSimple nameStr)
    | _ => none
  | _ => none

/--
Parse the manifest TSV at `path` and return the dual-key index.

Each line is `<olean-path>\t<imported-module-name>`. Lines that
fail to parse are silently skipped (the manifest may contain
non-olean lines from future schema extensions; today they don't).
-/
def loadIndex (path : System.FilePath) : IO ManifestIndex := do
  let content ← IO.FS.readFile path
  let mut modulePath : Std.HashMap Name System.FilePath := {}
  let mut imports : Std.HashMap Name (Array Name) := {}
  for line in content.splitOn "\n" do
    if line.isEmpty then continue
    match line.splitOn "\t" with
    | oleanPath :: importedStr :: _ =>
      match moduleNameOfPath oleanPath with
      | none => continue
      | some moduleName =>
        let importedName := Name.mkSimple importedStr
        modulePath := modulePath.insert moduleName (System.FilePath.mk oleanPath)
        let existing := imports.getD moduleName #[]
        imports := imports.insert moduleName (existing.push importedName)
    | _ => continue
  return { modulePath, imports }

/--
Transitive import closure starting from `seeds`. Returns the set of
all module names reachable from any seed (the seeds themselves are
included).

Modules not present in `idx.imports` are treated as leaves — typical
for stdlib modules (Init, Std, Lean) that are outside the Lake
workspace the manifest came from.

BFS over the imports graph; cycle-safe.
-/
def closure (idx : ManifestIndex) (seeds : Array Name) : Array Name := Id.run do
  let mut visited : Std.HashSet Name := {}
  let mut worklist : Array Name := seeds
  while !worklist.isEmpty do
    let some mod := worklist.back? | break
    worklist := worklist.pop
    if visited.contains mod then continue
    visited := visited.insert mod
    let directs := idx.imports.getD mod #[]
    for d in directs do
      if !visited.contains d then
        worklist := worklist.push d
  return visited.toArray

/--
Regex-extract `import X.Y.Z` declarations from a `.lean` source.

Lean's import section is well-defined: imports appear at the top of
the file, one per line, optionally interleaved with line comments
(`--`) or blank lines. The first non-import top-level statement
ends the import section.

Returns module names in the order they appear. Doesn't deduplicate
(caller's problem if it matters).
-/
def extractImports (path : System.FilePath) : IO (Array Name) := do
  let content ← IO.FS.readFile path
  let mut imports : Array Name := #[]
  for line in content.splitOn "\n" do
    -- Imports in Lean live at column 0 by convention. Requiring the
    -- literal "import " prefix avoids matching:
    --   * commented-out imports (`-- import Foo`) — first char is `-`
    --   * occurrences of the word "import" inside block comments
    --     (`/- some docstring discussing import -/`) — usually indented
    --   * identifiers starting with `import_*` — no trailing space
    if !line.startsWith "import " then continue
    let trimmed := (line.splitOn "--").head!  -- strip trailing line comments
    let words := trimmed.splitOn " " |>.filter (·.length > 0)
    match words with
    | "import" :: rest =>
      -- Module name is the first word that isn't an import-modifier
      -- keyword (`runtime`, `all`, etc., used by Lean's module system
      -- attributes; we don't care which flavor).
      if let some name := rest.find? (fun w => !w.isEmpty && !["runtime", "all", "open", "show"].contains w) then
        imports := imports.push (Name.mkSimple name)
    | _ => continue
  return imports

end RulesLean.Internal.Closure
