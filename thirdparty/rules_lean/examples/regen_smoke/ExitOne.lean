/-
Negative companion to ExitZero.lean. Returns 1 so a `lean_main_test`
wrapping it must fail. Not wired as a passing test in BUILD.bazel —
flip it manually (or add an `expect_test_failure`-style harness) to
confirm `lean_main_test` propagates non-zero exits.
-/

def main : IO UInt32 := do
  IO.println "lean_main_test smoke: exit 1"
  pure 1
