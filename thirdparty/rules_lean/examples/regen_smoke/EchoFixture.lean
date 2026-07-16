/-
Smoke for `lean_emit.data` (rules_lean 0.3.3+). Reads `fixture.txt`
from the action's work directory (staged via the `data` attr) and
echoes it verbatim. The diff_test verifies the echoed content matches
`fixture.txt`, proving the data file is reachable at its workspace-
relative path from the Lean entry.
-/

def main : IO Unit := do
  let s ← IO.FS.readFile "examples/regen_smoke/fixture.txt"
  IO.print s
