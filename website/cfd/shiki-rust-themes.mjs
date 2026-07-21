/**
 * Rust-native syntax themes for the CFD site.
 *
 * Base: Ayu, which rustdoc ships as one of its three built-in themes, so the
 * orange keywords and gold identifiers read as Rust rather than as generic
 * web code.
 *
 * Ayu's own backgrounds differ from this site's `--bg-2`, and several of its
 * token colours fall below WCAG AA against it. Comments fail worst in both
 * modes, and comments carry the explanation in these snippets. Rather than
 * pick a duller theme, each foreground is nudged along its own luminance axis
 * until it clears `MIN_RATIO`, which preserves the hue and fixes legibility.
 */
import { bundledThemes } from 'shiki';

/** `--bg-2` from tokens.css, the surface `pre.astro-code` is forced onto. */
const SURFACE = { dark: '#121a23', light: '#eef2f6' };
const MIN_RATIO = 4.5;

const srgb = (h) => [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16) / 255);
const lin = (v) => (v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4);
const lum = (h) => { const [r, g, b] = srgb(h).map(lin); return 0.2126 * r + 0.7152 * g + 0.0722 * b; };
const ratio = (a, b) => { const [x, y] = [lum(a), lum(b)]; return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05); };
const hex = (v) => Math.round(Math.min(255, Math.max(0, v * 255))).toString(16).padStart(2, '0');

/** Scale a colour toward white (dark mode) or black (light mode) until it clears AA. */
function legible(color, bg, mode) {
  if (!/^#[0-9a-f]{6}/i.test(color)) return color;
  const base = color.slice(0, 7);
  if (ratio(base, bg) >= MIN_RATIO) return color;
  const [r, g, b] = srgb(base);
  for (let step = 1; step <= 24; step++) {
    const t = step / 24;
    const mix = mode === 'dark'
      ? [r + (1 - r) * t, g + (1 - g) * t, b + (1 - b) * t]
      : [r * (1 - t), g * (1 - t), b * (1 - t)];
    const next = '#' + mix.map(hex).join('');
    if (ratio(next, bg) >= MIN_RATIO) return next + color.slice(7);
  }
  return mode === 'dark' ? '#ffffff' : '#000000';
}

const isComment = (scope) =>
  [].concat(scope ?? []).some((s) => String(s).startsWith('comment'));

async function derive(name, mode, commentColor) {
  const base = (await bundledThemes[name]()).default;
  const bg = SURFACE[mode];
  // Ayu carries its rules in `tokenColors`; other themes use `settings`. Read
  // whichever is populated, or an empty `settings` array silently drops every
  // rule and the block renders in one flat colour.
  const rules = base.tokenColors?.length ? base.tokenColors : (base.settings ?? []);
  const recoloured = rules.map((rule) => {
    const fg = rule.settings?.foreground;
    if (!fg) return rule;
    // Comments get an explicit colour: Ayu's grey reads as disabled text, and
    // these snippets carry their explanation in comments.
    const next = isComment(rule.scope) ? commentColor : legible(fg, bg, mode);
    return { ...rule, settings: { ...rule.settings, foreground: next } };
  });
  return {
    ...base,
    name: `rust-${mode}`,
    type: mode,
    bg,
    colors: { ...base.colors, 'editor.background': bg },
    tokenColors: recoloured,
    settings: recoloured,
  };
}

export const rustDark = await derive('ayu-dark', 'dark', '#7FA85F');
export const rustLight = await derive('ayu-light', 'light', '#4A7A2F');
