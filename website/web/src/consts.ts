/** Base URL of the standalone Starlight documentation site (website/docs),
 * served at its own Cloudflare Worker. The marketing site links out to it. */
export const DOCS_URL = 'https://docs.deepcausality.com';

/** GitHub repository. Subpaths (issues, discussions, blob/tree) are derived
 * from this base at the call site. */
export const GITHUB_URL = 'https://github.com/deepcausality-rs/deep_causality';

/** DeepCausality community Discord invite. */
export const DISCORD_URL = 'https://discord.gg/Bxj9P7JXSj';

/** Counterfactual Fluid Dynamics site (website/cfd), on its own Worker. */
export const CFD_URL = 'https://cfd.deepcausality.com';

/** Quantum causal models site (website/quantum), on its own Worker. */
export const QUANTUM_URL = 'https://quantum.deepcausality.com';

/** Deep Brain is a research programme rather than a crate, and it is written up
 * on the Center for Dynamic Causality's own research pages. There is no
 * deepcausality.com subdomain for it. */
export const DEEPBRAIN_URL = 'https://www.causalcenter.com/research/dynamic-knowledge/';

/** Steward of the project, and the home of the Deep Brain research programme. */
export const CAUSAL_CENTER_URL = 'https://www.causalcenter.com';

/** docs.rs base; the per-crate API reference is `${DOCSRS_BASE}/<crate>`. */
export const DOCSRS_BASE = 'https://docs.rs';
