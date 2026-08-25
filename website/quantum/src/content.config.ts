import { defineCollection } from 'astro:content';
import { z } from 'astro/zod';
import { glob } from 'astro/loaders';

/**
 * Content collections. Same shape as website/cfd: a `glob` loader over
 * `src/content/<collection>/<locale>/`, with the locale stripped from the
 * route in each `[...slug]` page.
 *
 * Frontmatter carries the structured facts a listing page needs; the MDX body
 * carries the walkthrough. Facts that appear on a listing live in frontmatter
 * so the index and the detail page cannot disagree.
 */

const examples = defineCollection({
  loader: glob({ pattern: '**/*.{md,mdx}', base: './src/content/examples' }),
  schema: z.object({
    title: z.string(),
    /** One-sentence question the run answers. */
    question: z.string(),
    summary: z.string(),
    /** Position: the two that exercise this crate come first. */
    order: z.number().default(0),
    command: z.string(),
    /**
     * Whether the example imports `deep_causality_quantum`. Five of the seven
     * examples in the package are quantum in subject but build on other crates,
     * and the listing says which is which rather than implying coverage.
     */
    usesCrate: z.boolean(),
    /** Items from this crate the example calls, when it calls any. */
    uses: z.array(z.string()).default([]),
    /** Workspace crates the example imports. */
    crates: z.array(z.string()).default([]),
    /** Repo-relative path to the example source. */
    source: z.string(),
  }),
});

export const collections = { examples };
