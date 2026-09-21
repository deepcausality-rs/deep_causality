## MODIFIED Requirements

### Requirement: The context hypergraph carries no symbolic dimension

The context types SHALL declare no `SYM` parameter and no `Symboid` arm. Their remaining parameters
are fixed by `context-associated-value-types` and `context-frame` rather than by this requirement,
which governs only the absence of the symbolic dimension.

The symbol node types `BaseSymbol` and `SymbolKind`, the `Symbolic` trait, the
`ContextoidType::Symboid` variant, the `ContextKind::Symboid` arm, the `symboid()` accessor and the
`SymbolicRepresentation` and `SymbolicResult` types SHALL NOT exist in the workspace.

Nothing ever constructed a symbol contextoid. Outside the enum's own match arms and a handful of
tests, the workspace contained no `ContextoidType::Symboid` construction at all, so the parameter
existed only to be threaded through signatures and filled in by type aliases. `Contextuable`,
`ContextuableGraph` and `ExtendableContextuableGraph` drop it with them.

#### Scenario: No symbolic parameter is declared

- **WHEN** a `Context`, `Contextoid` or `ContextoidType` is instantiated by hand
- **THEN** it accepts no symbolic type argument, and supplying one fails to compile

#### Scenario: The symbol surface is gone

- **WHEN** the workspace is searched for `BaseSymbol`, `SymbolKind`, `Symbolic`, `Symboid`,
  `SymbolicRepresentation` or `SymbolicResult`
- **THEN** none of them is declared or referenced

#### Scenario: The ready-made aliases absorb the change

- **WHEN** a consumer names `BaseContext`, `BaseContextoid`, `UniformContext` or
  `UniformContextoid`
- **THEN** the alias resolves without any edit at the call site, because the alias supplies the
  parameters
