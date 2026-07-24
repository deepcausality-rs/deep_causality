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

/** The crate is unpublished; this is the git-dependency line users need. */
export const CARGO_DEP =
  'deep_causality_cfd = { git = "https://github.com/deepcausality-rs/deep_causality.git", branch = "main" }';
