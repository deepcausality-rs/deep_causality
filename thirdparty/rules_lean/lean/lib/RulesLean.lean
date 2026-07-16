import RulesLean.Internal
import RulesLean.Olean
import RulesLean.Workspace

/-!
# RulesLean — umbrella import.

The library has two halves:

* `RulesLean.Olean` — introspect compiled `.olean` files using Lean's
  own `Lean.readModuleData` API (imports, exported constants, axiom
  usage).
* `RulesLean.Lake` — introspect Lake workspaces (manifests, package
  graph, cross-package deps) using Lake's own APIs.

Both halves are designed to be consumed from Bazel rules at
lake_workspace materialization time AND from standalone Lean tools
that just want structured information about a Lean codebase.

CLI entry points (mostly thin shells over the library API) live in
`RulesLean.Cli.*` and are built as separate `lean_exe` targets in the
lakefile.
-/
