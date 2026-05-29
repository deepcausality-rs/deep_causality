/** Base URL of the standalone Starlight documentation site (website/docs),
 * served at its own Cloudflare Worker. The marketing site links out to it. */
export const DOCS_URL = 'https://docs.deepcausality.com';

/** GitHub repository. Subpaths (issues, discussions, blob/tree) are derived
 * from this base at the call site. */
export const GITHUB_URL = 'https://github.com/deepcausality-rs/deep_causality';

/** DeepCausality community Discord invite. */
export const DISCORD_URL = 'https://discord.gg/Bxj9P7JXSj';

/** docs.rs base; the per-crate API reference is `${DOCSRS_BASE}/<crate>`. */
export const DOCSRS_BASE = 'https://docs.rs';
