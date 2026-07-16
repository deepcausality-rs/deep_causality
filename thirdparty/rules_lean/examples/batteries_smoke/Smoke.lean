/-
Smoke test for rules_lean + lake_workspace, end-to-end:
  1. The repository rule downloaded the Lean toolchain.
  2. `lake update` resolved Batteries at the manifest-pinned rev.
  3. `lake build batteries` produced oleans under .lake/packages/batteries/...
  4. `lean_prebuilt_library` exposed them, and our lean_test compiled
     this file against `@lake_deps_smoke//:batteries`.

If the import resolves, the whole chain works. We use Batteries instead
of Mathlib so CI doesn't pull a multi-GB cache; deliberately avoid
calling specific Batteries APIs (they churn across releases) so the test
gates the *plumbing*, not the library surface.
-/

import Batteries.Data.HashMap.Basic

example : True := trivial
