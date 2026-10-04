// Copy the weather example's committed run files into public/traces/, where the compositions read
// them, and the design system's fonts into public/fonts/. The copies are generated; the example's
// own files and the website's fonts stay the single source.
import { copyFileSync, mkdirSync, readdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const example = join(here, '../../../../examples/avionics_examples/cfd/plasma_blackout/weather');
const out = join(here, '../public/traces');
mkdirSync(out, { recursive: true });
for (const file of ['weather_trace.csv', 'weather_draws.csv', 'weather_table.csv', 'output.txt']) {
  copyFileSync(join(example, file), join(out, file));
  console.log(`synced ${file}`);
}
// The audit log of each world's reference draw, which names the world that replaced the baseline.
mkdirSync(join(out, 'audit'), { recursive: true });
for (const file of readdirSync(join(example, 'audit')).filter((f) => f.endsWith('.draw-0.log'))) {
  copyFileSync(join(example, 'audit', file), join(out, 'audit', file));
  console.log(`synced audit/${file}`);
}

const fonts = join(here, '../../../../website/web/public/fonts');
const fontsOut = join(here, '../public/fonts');
mkdirSync(fontsOut, { recursive: true });
for (const file of ['geist-latin.woff2', 'geist-latin-ext.woff2', 'jetbrains-mono-latin.woff2', 'jetbrains-mono-latin-ext.woff2']) {
  copyFileSync(join(fonts, file), join(fontsOut, file));
}
console.log('synced fonts');
