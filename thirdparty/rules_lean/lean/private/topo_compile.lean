/-
rules_lean topo-compile driver.

Compiles a set of staged `.lean` modules to `.olean` in import-topological
order, optionally publishes each olean to a declared output, and optionally
`--run`s an entry module (capturing stdout for `lean_emit`, or propagating the
exit code for `lean_main_test`). Replaces the `run_shell` `tsort` pipeline so
every rule drives the compiler through `ctx.actions.run` — no shell. The only
subprocess spawned is the `lean` compiler itself; all staging/copying uses
native `IO.FS`.

Self-contained (Lean core only — no imports) so it can be invoked as
`lean --run topo_compile.lean <manifest>` with just the toolchain present.
String handling uses `toList`/`ofList` rather than the Slice-returning
`trim`/`drop`/`dropRight`, for stability across the 4.30 String API.

Manifest: one tab-separated directive per line. Relative paths are resolved
against the process cwd (the exec root), so they work whether the action runs
at the exec root or the run step `cwd`s into the work dir.
  lean      <lean compiler binary>
  work      <writable scratch root to create>
  leanpath  <dir>                 (0+; disjoint-namespace dep roots)
  stage     <file>\t<rel>         (0+; copied to <work>/<rel>, not compiled)
  module    <rel>                 (1+; a staged .lean to compile, topo-ordered)
  output    <rel>\t<dest .olean>  (0+; publish a compiled module's olean)
  marker    <dest>                (0/1; sentinel file to write)
  entry     <rel>                 (0/1; after compiling, `--run` this module)
  stdout    <dest>                (0/1; with `entry`, capture its stdout here)
-/

private def fp (s : String) : System.FilePath := ⟨s⟩

private def isWs (c : Char) : Bool := c == ' ' || c == '\t' || c == '\r'

private def trimStr (s : String) : String :=
  String.ofList (((s.toList.dropWhile isWs).reverse.dropWhile isWs).reverse)

/-- Native file copy (no `cp`): create the parent dir, then byte-copy. -/
private def copyFile (src dst : String) : IO Unit := do
  match (fp dst).parent with
  | some p => IO.FS.createDirAll p
  | none => pure ()
  IO.FS.writeBinFile (fp dst) (← IO.FS.readBinFile (fp src))

private def stripDotLean (s : String) : String :=
  if s.endsWith ".lean" then String.ofList (s.toList.take (s.toList.length - 5)) else s

private def modName (rel : String) : String := (stripDotLean rel).replace "/" "."

private def oleanRel (rel : String) : String := stripDotLean rel ++ ".olean"

/-- Module names imported by a source (first token of each `import …` line). -/
private def parseImports (src : String) : List String :=
  (src.splitOn "\n").filterMap fun line =>
    let t := trimStr line
    if t.startsWith "import " then
      match (trimStr (String.ofList (t.toList.drop 7))).splitOn " " |>.head? with
      | some m => if m.isEmpty then none else some m
      | none => none
    else none

/-- Kahn topological sort: emit nodes whose in-set imports are already emitted.
    Edges to modules outside `mods` are ignored (those deps come via LEAN_PATH). -/
private partial def topoLoop (mods : List String) (impsOf : String → List String)
    (remaining done acc : List String) : Except String (List String) :=
  if remaining.isEmpty then .ok acc
  else
    let ready := remaining.filter fun m =>
      (impsOf m).all fun d => done.contains d || !mods.contains d
    if ready.isEmpty then .error "import cycle among lean srcs"
    else
      topoLoop mods impsOf
        (remaining.filter fun m => !ready.contains m) (done ++ ready) (acc ++ ready)

structure Job where
  leanBin : String := ""
  work : String := ""
  marker : String := ""
  entry : Option String := none
  stdout : Option String := none
  leanPaths : List String := []
  stages : List (String × String) := []   -- (src, rel)
  modules : List String := []              -- rels to compile
  outputs : List (String × String) := []   -- (rel, dest olean)

private def parseManifest (content : String) : Job := Id.run do
  let mut j : Job := {}
  for line in content.splitOn "\n" do
    match line.splitOn "\t" with
    | ["lean", v] => j := { j with leanBin := v }
    | ["work", v] => j := { j with work := v }
    | ["marker", v] => j := { j with marker := v }
    | ["entry", v] => j := { j with entry := some v }
    | ["stdout", v] => j := { j with stdout := some v }
    | ["leanpath", v] => j := { j with leanPaths := j.leanPaths ++ [v] }
    | ["stage", src, rel] => j := { j with stages := j.stages ++ [(src, rel)] }
    | ["module", rel] => j := { j with modules := j.modules ++ [rel] }
    | ["output", rel, dst] => j := { j with outputs := j.outputs ++ [(rel, dst)] }
    | _ => pure ()
  return j

def main (args : List String) : IO UInt32 := do
  let some manifest := args.head?
    | do IO.eprintln "topo_compile: usage: topo_compile <manifest>"; return 1
  let j := parseManifest (← IO.FS.readFile (fp manifest))

  -- Resolve relative paths against the exec root, so the `--run` step (which
  -- cds into the work dir) still finds the toolchain + dep roots.
  let root := (← IO.currentDir).toString
  let abs := fun (p : String) => if p.startsWith "/" then p else root ++ "/" ++ p
  let work := abs j.work
  let leanBin := abs j.leanBin
  let leanPath := String.intercalate ":" (work :: j.leanPaths.map abs)

  -- 1. Stage srcs + same-namespace dep oleans + data into the work root.
  IO.FS.createDirAll (fp work)
  for (src, rel) in j.stages do
    copyFile (abs src) (work ++ "/" ++ rel)

  -- 2. Read each module's imports, then topo-sort.
  let mut impMap : List (String × List String) := []
  for rel in j.modules do
    impMap := impMap ++ [(modName rel, parseImports (← IO.FS.readFile (fp (work ++ "/" ++ rel))))]
  let impsOf := fun m => (impMap.find? (·.1 == m)).map (·.2) |>.getD []
  let mods := j.modules.map modName

  match topoLoop mods impsOf mods [] [] with
  | .error e => do IO.eprintln s!"topo_compile: {e}"; return 1
  | .ok order =>
    let nameToRel := j.modules.map fun rel => (modName rel, rel)
    -- 3. Compile in dependency order (abs paths, so cwd is irrelevant here).
    for m in order do
      if let some (_, rel) := nameToRel.find? (·.1 == m) then
        let r ← IO.Process.output {
          cmd := leanBin
          args := #["--root=" ++ work, "-o", work ++ "/" ++ oleanRel rel, work ++ "/" ++ rel]
          env := #[("LEAN_PATH", some leanPath)]
        }
        if r.exitCode != 0 then
          IO.eprint r.stdout; IO.eprint r.stderr
          IO.eprintln s!"topo_compile: lean failed on {rel}"
          return 1

    -- 4. Publish declared oleans.
    for (rel, dst) in j.outputs do
      copyFile (work ++ "/" ++ oleanRel rel) (abs dst)

    -- 5. Optionally `--run` the entry (cwd = work, so its relative file opens
    --    resolve to staged data). Capture stdout to a file, or pass it through.
    match j.entry with
    | none => pure ()
    | some entry =>
      let r ← IO.Process.output {
        cmd := leanBin
        args := #["--root=" ++ work, "--run", entry]
        cwd := work
        env := #[("LEAN_PATH", some leanPath)]
      }
      if r.exitCode != 0 then
        IO.eprint r.stdout; IO.eprint r.stderr
        IO.eprintln s!"topo_compile: entry {entry} failed"
        return r.exitCode
      match j.stdout with
      | some dst => IO.FS.writeFile (fp (abs dst)) r.stdout
      | none => IO.print r.stdout

    -- 6. Marker written LAST: a full-success sentinel (compile [+ run] all OK).
    if !j.marker.isEmpty then
      IO.FS.writeFile (fp (abs j.marker)) "rules_lean ok\n"
    return 0
