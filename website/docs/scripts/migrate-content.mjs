/*
 * One-shot content migration: website/web docs -> Starlight docs app.
 *
 * Transforms applied per file:
 *  - frontmatter: keep title/description; drop `section:`; map `order: N`
 *    to Starlight's `sidebar.order`.
 *  - body links: `/docs/...` -> root-relative `/...` (same docs origin);
 *    `/examples/...` and `/blog/...` and bare `/` -> absolute www URLs
 *    (those stay on the marketing site).
 *
 * Idempotent: safe to re-run; it overwrites the destination files.
 */
import { readdirSync, mkdirSync, readFileSync, writeFileSync, statSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const SRC = join(here, '../../web/src/content/docs');
const DEST = join(here, '../src/content/docs');
const WWW = 'https://www.deepcausality.com';
const sections = ['overview', 'getting-started', 'concepts'];

// Files hand-edited after migration that the script must NOT overwrite on a
// re-run. overview/why.md had its two mermaid diagrams converted to static,
// no-JS inline SVG; re-copying from source would revert them.
const SKIP = new Set(['overview/why.md']);

function transformFrontmatter(fm) {
  const out = [];
  for (const line of fm.split('\n')) {
    if (/^section:\s/.test(line)) continue; // drop
    const m = line.match(/^order:\s*(.+)$/);
    if (m) {
      out.push('sidebar:');
      out.push(`  order: ${m[1].trim()}`);
      continue;
    }
    out.push(line);
  }
  return out.join('\n');
}

function transformBody(body) {
  return body
    .replaceAll('](/docs/', '](/')
    .replaceAll('](/examples/', `](${WWW}/examples/`)
    .replaceAll('](/blog/', `](${WWW}/blog/`)
    .replaceAll('](/)', `](${WWW}/)`);
}

function transformFile(text) {
  const m = text.match(/^---\n([\s\S]*?)\n---\n?([\s\S]*)$/);
  if (!m) return transformBody(text); // no frontmatter
  const [, fm, body] = m;
  return `---\n${transformFrontmatter(fm)}\n---\n${transformBody(body)}`;
}

let count = 0;
for (const section of sections) {
  const srcDir = join(SRC, section);
  const destDir = join(DEST, section);
  mkdirSync(destDir, { recursive: true });
  for (const name of readdirSync(srcDir)) {
    const srcPath = join(srcDir, name);
    if (!statSync(srcPath).isFile()) continue;
    if (!/\.(md|mdx)$/.test(name)) continue;
    if (SKIP.has(`${section}/${name}`)) { console.log(`  (skip, hand-edited) ${section}/${name}`); continue; }
    const out = transformFile(readFileSync(srcPath, 'utf8'));
    writeFileSync(join(destDir, name), out);
    count++;
    console.log(`  ${section}/${name}`);
  }
}
console.log(`Migrated ${count} files.`);
