import RulesLean.Internal.AxiomDeps
import RulesLean.Internal.Closure
import RulesLean.Internal.Filesystem

/-!
# `RulesLean.Internal.*` — unstable surface.

Everything under `RulesLean.Internal` is **not part of the
stable API**. Names, signatures, and semantics here may change
between any two `rules_lean` releases — even patch bumps.

The two halves of the stable surface are:

* `RulesLean.Olean` — olean introspection (imports, exported
  constants).
* `RulesLean.Workspace` — Lake workspace introspection (manifests,
  namespace→package index).

Use `Internal.*` if you need a primitive the stable surface doesn't
expose — but pin the rules_lean version you depend on, and expect
to revisit usages on every upgrade. We add a deprecation warning
to anything we move into stable; we don't promise the same for
internal-to-internal renames.

## What lives here

* `Internal.Filesystem` — directory-walking + olean-tree helpers
  used by the workspace-introspection code. Internal because the
  set of helpers we expose changes as `Workspace`'s needs evolve.
* `Internal.AxiomDeps` — axiom-dependency tools. Currently exposes
  `declaredAxioms` (modules' own axiom declarations) and `isAxiom`
  (presence check). The deeper "transitive axiom closure across
  the import graph" lands here once a consumer pushes on the shape.
-/
