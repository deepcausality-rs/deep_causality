import { defineCollection } from 'astro:content';
import { z } from 'astro/zod';
import { glob } from 'astro/loaders';

const blog = defineCollection({
  loader: glob({ pattern: '**/*.{md,mdx}', base: './src/content/blog' }),
  schema: z.object({
    title: z.string(),
    date: z.coerce.date(),
    author: z.string().default('DeepCausality'),
    tags: z.array(z.string()).default([]),
    description: z.string().optional(),
    /** Per-post social share image. Path relative to site root, e.g. /img/og/foo.jpg. Falls back to the site default. */
    image: z.string().optional(),
    /** Optional ISO date for last meaningful update. Defaults to `date` for JSON-LD/article:modified_time. */
    updated: z.coerce.date().optional(),
    draft: z.boolean().default(false),
  }),
});

const examples = defineCollection({
  loader: glob({ pattern: '**/*.{md,mdx}', base: './src/content/examples' }),
  schema: z.object({
    title: z.string(),
    domain: z.string(),
    summary: z.string(),
    crates: z.array(z.string()).default([]),
    order: z.number().default(0),
    /** Top-level grouping used by the Examples dropdown and the per-category pages. */
    category: z.enum(['foundations', 'aerospace', 'physics', 'medicine', 'mathematics']),
    /**
     * Repo-relative path to the example source, e.g. `examples/starter_example`.
     * `ExampleDetail` renders a "Source:" link when present and omits it when
     * absent. No content sets it yet, so the link does not currently render —
     * the field exists because `[...slug].astro` already reads it.
     */
    source: z.string().optional(),
    further: z
      .array(z.object({ label: z.string(), href: z.string() }))
      .default([]),
  }),
});

export const collections = { blog, examples };
