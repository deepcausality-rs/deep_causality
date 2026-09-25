# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/deepcausality-rs/deep_causality/releases/tag/deep_causality_context-v0.1.0) - 2026-09-23

### Added

- *(deep_causality_context_store)* [**breaking**] membership events carry the relations to members
- *(deep_causality_context_store)* [**breaking**] atomic ContextStorage::commit; per-node idempotent Substrate::deposit
- *(deep_causality_context)* add ContextStore, StoreError, Context::apply and subscribe
- *(deep_causality_context)* add Storable, the record projection, snapshot and restore
- *(deep_causality_context)* add Storable, the record projection, snapshot and restore
- *(deep_causality_context)* [**breaking**] add the in-memory backend and name the extra contexts
- *(deep_causality_context_store)* add the persistence contract crate and move the context vocabulary into it
- *(deep_causality_context)* spacetime nodes report their own metric signature
- *(deep_causality_context)* [**breaking**] add ContextFrame, NoSpaceTime and MetricFamily; declare FloatType per crate
- *(context)* [**breaking**] store the vertical datum on GeoSpace, drop the welded uncertain aliases

### Fixed

- *(deep_causality_context)* a store event never reaches a local extra context
- *(deep_causality_context)* Applied a number of fixes

### Other

- *(deep_causality_context)* restore builds stored extras directly; scope the local-extra rule to container events
- *(deep_causality_context)* tighten context-store tests after review
- edit all READMEs for clarity, concision and correctness
- *(deep_causality_context_store)* document the contract, re-derive the tier block, add SBOMs
- *(deep_causality_context)* Improved tests and test coverage.
- *(context)* [**breaking**] consolidate the context signature
- *(deep_causality_context)* [**breaking**] make the scalar a parameter on every context node type
- *(context)* [**breaking**] merge MinkowskiSpacetime into LorentzianSpacetime, add edge read accessors
- *(context)* [**breaking**] own the context identifiers, store the edge relation, drop QuaternionSpace
- *(context)* [**breaking**] replace the value parameters with associated types
- *(context)* [**breaking**] make context an opt-in crate, drop the symbolic dimension
- *(context)* [**breaking**] make context an opt-in crate, drop the symbolic dimension
- *(deep_causality_context)* move the context layer to deep_causality_context
