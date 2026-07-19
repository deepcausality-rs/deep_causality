# 04 — Surface, border, elevation

## The governing idea

There is no soft-card stack on this site. Grouped content is bounded by a 1px
hairline, not by a shadow. Drop shadows are reserved for elements that genuinely
float above the page: popovers, the search panel, the mobile menu sheet.

The reason is the brand anchor. The hero art is a line drawing of a network.
Hairlines continue that language. Blurred shadow edges contradict it.

## Radii

From `tokens.css:118-122`.

| Token | Value | Use |
|---|---|---|
| `--radius-sm` | 4px | Default control, inline code, tags, buttons |
| `--radius-md` | 10px | Cards, code blocks, panels |
| `--radius-lg` | 16px | Hero plate. One per page at most |
| `--radius-pill` | 999px | Tag rows on example detail pages |

`--radius-sm` also serves as the focus-ring radius in `global.css:43`, so the
ring follows the shape of whatever it wraps.

## Surface ladder

Depth is expressed by background steps rather than by shadow. Each step is small,
which is what keeps the page reading as one field rather than as stacked panes.

| Layer | Token | Dark | Light |
|---|---|---|---|
| Page | `--bg-0` | `#070b10` | `#ffffff` |
| Section / raised | `--bg-1` | `#0b1118` | `#f6f8fa` |
| Code / input | `--bg-2` | `#121a23` | `#eef2f6` |
| Hover / selected / inline code | `--bg-3` | `#1a2430` | `#e2e8ef` |

Note the direction reverses between themes. Dark mode raises a surface by making
it lighter; light mode raises it by making it darker. Both move away from the
page background, so a component written against the tokens works in both without
a per-theme branch.

## Borders

| Token | Dark | Light | Use |
|---|---|---|---|
| `--line-1` | `#1f2a36` | `#d8dee4` | Default border, hairline divider, section rule |
| `--line-2` | `#2a3a4a` | `#c0c8d1` | Hover border, focused input, emphasis |

The standard container is `border: 1px solid var(--line-1)` plus `--shadow-1`.
The standard hover is a promotion to `--line-2`. That single-step border change
carries most of the interactive feedback on this site, which is why it can afford
to skip background transitions entirely.

## Shadows

Theme-aware, so a shadow always sits in the page background's colour family.

Dark, from `tokens.css:36-39`:

```css
--shadow-1: inset 0 1px 0 0 rgba(255, 255, 255, 0.04);   /* hairline lift */
--shadow-2: 0 12px 24px -16px rgba(8, 16, 24, 0.70);     /* floating panel */
--shadow-3: 0 24px 48px -24px rgba(8, 16, 24, 0.80);     /* modal */
```

Light, from `tokens.css:62-64`:

```css
--shadow-1: inset 0 1px 0 0 rgba(255, 255, 255, 0.6);
--shadow-2: 0 10px 24px -16px rgba(15, 23, 32, 0.18);
--shadow-3: 0 24px 48px -24px rgba(15, 23, 32, 0.22);
```

Two details worth preserving when adding a shadow.

`--shadow-1` is an inset, not a drop. It draws a single bright line along the top
edge of a surface, which reads as a lit lip rather than as a floating object. It
is the default companion to a hairline border.

`--shadow-2` and `--shadow-3` carry a large negative spread (`-16px`, `-24px`)
against a comparable blur. That pulls the shadow in under the element so it never
leaks a grey halo sideways. Copying the offset and blur while dropping the spread
produces the generic SaaS glow the system is built to avoid.

## Accent halos

Three wide, very soft accent glows ship outside the token set. They are not
"outer-glow on a button"; the large negative spread keeps them behind the
element as ambient light rather than as a rim.

```css
/* Hero.astro:120 — hero art frame, layered over --shadow-2 */
0 0 80px -28px color-mix(in srgb, var(--accent) 38%, transparent)
/* ExampleCard.astro:66 — card hover */
0 0 40px -22px color-mix(in srgb, var(--accent) 32%, transparent)
/* Hero.astro:92-93 — primary button lip */
inset 0 1px 0 0 color-mix(in srgb, var(--accent) 60%, black),
0 1px 2px 0 color-mix(in srgb, var(--accent) 50%, transparent)
```

Blur is 80px at the hero and 40px on a card, scaled to the element. Spread is
roughly a third of blur, negative. Copy those proportions if a new halo is
needed. None of the three is tokenized.

## Banned

- `rounded-2xl` plus `shadow-md` soft-card stacks, especially in rows of three.
- Glassmorphism. No `backdrop-filter: blur(...)` over the hero or the sidebar.
  One exception ships: a 2px blur on the mobile menu scrim,
  `SiteHeader.astro:298-299`. See [08-drift.md](08-drift.md) §8.
- Outer-glow shadows on buttons.
- Purple or magenta shadow tint.
- Animating `box-shadow` on a large surface. See [05-motion.md](05-motion.md).

## Adding a surface

Ask in this order:

1. Does a background step alone separate it? Use `--bg-1`/`--bg-2`.
2. Does it need an edge? Add `border: 1px solid var(--line-1)`.
3. Does it need to feel lit? Add `--shadow-1`.
4. Does it genuinely float above the page in z, and is it dismissible? Only then
   reach for `--shadow-2` or `--shadow-3`.

Most components stop at step 3.
