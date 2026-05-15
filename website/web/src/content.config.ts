import { defineCollection, z } from 'astro:content';
import { glob } from 'astro/loaders';

const blog = defineCollection({
  loader: glob({ pattern: '**/*.{md,mdx}', base: './src/content/blog' }),
  schema: z.object({
    title: z.string(),
    date: z.coerce.date(),
    author: z.string().default('DeepCausality'),
    tags: z.array(z.string()).default([]),
    summary: z.string().optional(),
    draft: z.boolean().default(false),
  }),
});

const docs = defineCollection({
  loader: glob({ pattern: '**/*.{md,mdx}', base: './src/content/docs' }),
  schema: z.object({
    title: z.string(),
    description: z.string().optional(),
    order: z.number().default(0),
    section: z.enum(['getting-started', 'concepts', 'guides', 'reference', 'monograph']),
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
  }),
});

const monograph = defineCollection({
  loader: glob({ pattern: '**/*.{md,mdx}', base: './src/content/monograph' }),
  schema: z.object({
    title: z.string(),
    volume: z.enum(['epp', 'metaphysics', 'formalization', 'ontology', 'epistemology', 'teleology']),
    pdf: z.string().optional(),
    summary: z.string().optional(),
  }),
});

export const collections = { blog, docs, examples, monograph };
