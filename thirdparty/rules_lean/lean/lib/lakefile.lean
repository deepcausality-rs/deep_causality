import Lake
open Lake DSL

/-!
# `RulesLean` — Lean library shipped alongside the Bazel rules.

This is the Lake project for the `RulesLean` Lean library. Built
independently of the Bazel rules with `lake build` (from `lean/lib/`),
or compiled at lake_workspace materialization time when consumers
register the library via the upcoming `lean_library_source` rule (a
follow-up to the v0.3 work).

Public API: `RulesLean.Olean` and `RulesLean.Lake` — see those modules
for what's exported.

CLI entry points (built as `lake build oleanImports`, etc.) live under
`RulesLean.Cli.*`.

Toolchain pin: `lean-toolchain` next to this file. CI runs `lake build`
+ `lake test` against this version; consumers can build the library
against any Lean version their `lake_workspace` materializes — the
library uses only stable Lean + Lake APIs.
-/

package «RulesLean» where

@[default_target]
lean_lib RulesLean where

lean_exe oleanImports where
  root := `RulesLean.Cli.OleanImports
  supportInterpreter := true

lean_exe leanClosure where
  root := `RulesLean.Cli.LeanClosure
  supportInterpreter := true
