// Copy the retropropulsion example's committed run files, and the weather table it reads in flight,
// into public/traces/, where the compositions read them. The copies are generated; the examples'
// own files stay the single source.
import { copyFileSync, mkdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const examples = join(here, '../../../../examples/avionics_examples/cfd/plasma_blackout');
const out = join(here, '../public/traces');
mkdirSync(out, { recursive: true });
const files = [
  ['retropropulsion', 'retropropulsion_trace.csv'],
  ['retropropulsion', 'retropropulsion_branch_trace.csv'],
  ['retropropulsion', 'retropropulsion_uninformed_trace.csv'],
  ['retropropulsion', 'retropropulsion_branches.csv'],
  ['retropropulsion', 'output.txt'],
  ['weather', 'weather_table.csv'],
];
for (const [dir, file] of files) {
  copyFileSync(join(examples, dir, file), join(out, file));
  console.log(`synced ${dir}/${file}`);
}

// The design system's fonts, from the project website, the single copy in the repository.
// The end card's logos, from the repository's img/.
const logos = join(here, '../../../../img');
const logosOut = join(here, '../public/logos');
mkdirSync(logosOut, { recursive: true });
for (const file of ['logo_background.jpg', 'causal_center_logo_dark.svg']) {
  copyFileSync(join(logos, file), join(logosOut, file));
}
console.log('synced logos');

const fonts = join(here, '../../../../website/web/public/fonts');
const fontsOut = join(here, '../public/fonts');
mkdirSync(fontsOut, { recursive: true });
for (const file of ['geist-latin.woff2', 'geist-latin-ext.woff2', 'jetbrains-mono-latin.woff2', 'jetbrains-mono-latin-ext.woff2']) {
  copyFileSync(join(fonts, file), join(fontsOut, file));
}
console.log('synced fonts');
