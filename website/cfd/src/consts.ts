/** The DeepCausality project website. */
export const SITE_URL = 'https://www.deepcausality.com';

/** Starlight documentation site for the core library. */
export const DOCS_URL = 'https://docs.deepcausality.com';

/** GitHub repository. Subpaths are derived from this base at the call site. */
export const GITHUB_URL = 'https://github.com/deepcausality-rs/deep_causality';

/** Source tree roots referenced repeatedly by blueprints and validation pages. */
export const CRATE_URL = `${GITHUB_URL}/tree/main/deep_causality_cfd`;
export const VERIFICATION_URL = `${CRATE_URL}/verification`;
export const STUDIES_URL = `${CRATE_URL}/studies`;
export const EXAMPLES_URL = `${GITHUB_URL}/tree/main/examples/avionics_examples/cfd`;

/** DeepCausality community Discord invite. */
export const DISCORD_URL = 'https://discord.gg/Bxj9P7JXSj';

/** Released on crates.io since 2026-08-12. */
export const CARGO_ADD = 'cargo add deep_causality_cfd';

/** Git dependency, for work that has not been released yet. */
export const CARGO_DEP =
  'deep_causality_cfd = { git = "https://github.com/deepcausality-rs/deep_causality.git", branch = "main" }';

/**
 * The machine every measured figure on this site was taken on. Wall clocks are
 * unreadable without it: a reader cannot calibrate a number without knowing
 * whether it came off a laptop or a workstation. Quote this string, do not
 * paraphrase it, and do not attach it to a figure measured elsewhere.
 */
export const MACHINE = 'Apple M3 Max, 16 cores (12 performance + 4 efficiency), 128 GB';

/** Steward of the DeepCausality project. */
export const MAINTAINER = 'Center for Dynamic Causality';
export const MAINTAINER_URL = 'https://www.causalcenter.com';
