/-
Smoke entry for `lean_regen_test`. `main` prints a fixed string that
the BUILD file diffs against `expected.txt` — proving the
lean_emit + skylib diff_test pipeline runs end-to-end.
-/

def main : IO Unit := IO.println "hello from lean_regen_test"
