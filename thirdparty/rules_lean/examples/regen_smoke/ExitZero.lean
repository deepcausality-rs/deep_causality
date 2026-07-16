/-
Smoke for `lean_main_test`. Lean main returns 0 → the wrapping
`lean_main_test` rule passes. The variant `ExitOne.lean` returns 1
to confirm non-zero exit causes a test failure (verified by an
`expect_test_failure`-style negative test, omitted here for now).
-/

def main : IO UInt32 := do
  IO.println "lean_main_test smoke: exit 0"
  pure 0
