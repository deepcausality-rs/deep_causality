import { defineCollection } from 'astro:content';
import { z } from 'astro/zod';
import { glob } from 'astro/loaders';

/**
 * Content collections. Same shape as website/web: a `glob` loader over
 * `src/content/<collection>/<locale>/`, with the locale stripped from the
 * route in each `[...slug]` page.
 *
 * Frontmatter carries the structured facts a listing page needs; the MDX body
 * carries the walkthrough. Numbers that appear on a listing live in
 * frontmatter so the index and the detail page cannot disagree.
 */

const blueprints = defineCollection({
  loader: glob({ pattern: '**/*.{md,mdx}', base: './src/content/blueprints' }),
  schema: z.object({
    title: z.string(),
    /** The job, phrased as a task. Shown on the index. */
    task: z.string(),
    summary: z.string(),
    order: z.number().default(0),
    /** DSL verbs the blueprint exercises. */
    verbs: z.array(z.string()).default([]),
    /** Worked examples that use this blueprint. */
    worked: z.array(z.object({ label: z.string(), href: z.string() })).default([]),
  }),
});

const examples = defineCollection({
  loader: glob({ pattern: '**/*.{md,mdx}', base: './src/content/examples' }),
  schema: z.object({
    title: z.string(),
    /** One-sentence engineering question the run answers. */
    question: z.string(),
    summary: z.string(),
    /** Position on the capability ladder, not alphabetical. */
    order: z.number().default(0),
    command: z.string(),
    /** DSL surface exercised. */
    uses: z.array(z.string()).default([]),
    runtime: z.string(),
    /** Headline figures, quoted verbatim from the committed output.txt. */
    results: z.array(z.string()).default([]),
    limitation: z.string(),
    /** False when no output.txt is committed; the page then says so. */
    recorded: z.boolean().default(true),
    /** Repo-relative path to the example source. */
    source: z.string(),
  }),
});

export const collections = { blueprints, examples };
