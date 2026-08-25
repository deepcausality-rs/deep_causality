/** The DeepCausality project website. */
export const SITE_URL = 'https://www.deepcausality.com';

/** Starlight documentation site for the core library. */
export const DOCS_URL = 'https://docs.deepcausality.com';

/** Sibling crate site: computational fluid dynamics. */
export const CFD_URL = 'https://cfd.deepcausality.com';

/** GitHub repository. Subpaths are derived from this base at the call site. */
export const GITHUB_URL = 'https://github.com/deepcausality-rs/deep_causality';

/** Source tree roots referenced repeatedly across the layer pages. */
export const CRATE_URL = `${GITHUB_URL}/tree/main/deep_causality_quantum`;
export const SRC_URL = `${CRATE_URL}/src`;
export const TESTS_URL = `${CRATE_URL}/tests`;
export const PAPERS_URL = `${CRATE_URL}/papers`;
export const EXAMPLES_URL = `${GITHUB_URL}/tree/main/examples/quantum_examples`;
export const LEAN_URL = `${GITHUB_URL}/tree/main/lean/DeepCausalityFormal/Quantum`;
export const THEOREM_MAP_URL = `${GITHUB_URL}/blob/main/lean/THEOREM_MAP.md`;
export const LEAN_STATUS_URL = `${CRATE_URL}/LEAN_QUANTUM.md`;

/** The carrier types this crate builds on, which live in sibling crates. */
export const MULTIVECTOR_URL = `${GITHUB_URL}/tree/main/deep_causality_multivector`;
export const METRIC_URL = `${GITHUB_URL}/tree/main/deep_causality_metric`;
export const TENSOR_URL = `${GITHUB_URL}/tree/main/deep_causality_tensor`;

/** DeepCausality community Discord invite. */
export const DISCORD_URL = 'https://discord.gg/Bxj9P7JXSj';

/**
 * The crate carries `publish = false`, so there is no crates.io release and no
 * `cargo add` line. A git dependency is the only way to take it today; quote
 * this constant rather than writing a version number that does not exist.
 */
export const CARGO_DEP =
  'deep_causality_quantum = { git = "https://github.com/deepcausality-rs/deep_causality.git", branch = "main" }';

/** Workspace MSRV, from `rust-version` in the root Cargo.toml. */
export const MSRV = '1.97.1';

/** Steward of the DeepCausality project. */
export const MAINTAINER = 'Center for Dynamic Causality';
export const MAINTAINER_URL = 'https://www.causalcenter.com';
