# 05 — Motion

Motion intensity sits at roughly 6 on a 1–10 dial. Fluid CSS, no scroll-jacking,
no animation library.

## Tokens

From `tokens.css:124-129`.

| Token | Value | Use |
|---|---|---|
| `--dur-fast` | 120ms | Hover, focus, link underline |
| `--dur-med` | 220ms | Card hover, nav reveal, theme swap |
| `--dur-slow` | 420ms | Section entrance |
| `--ease-out` | `cubic-bezier(0.16, 1, 0.3, 1)` | Almost everything |
| `--ease-in-out` | `cubic-bezier(0.65, 0, 0.35, 1)` | Symmetric two-way motion |

`--ease-out` is a strong decelerating curve. It moves most of the distance early,
then settles. Applied to a 2px hover lift it reads as a physical snap rather than
as a slide.

## Reduced motion

The contract is enforced at the token level, which is the important architectural
decision here. `tokens.css:135-141`:

```css
@media (prefers-reduced-motion: reduce) {
  :root {
    --dur-fast: 0ms;
    --dur-med: 0ms;
    --dur-slow: 0ms;
  }
}
```

Because every duration resolves through a token, a component that writes
`transition: transform var(--dur-med) var(--ease-out)` is already compliant. It
needs no media query of its own.

This inverts the usual rule. `DESIGN.md` §6 asks every transition to sit inside
`@media (prefers-reduced-motion: no-preference)`. The token override makes that
wrapper redundant for any transition whose duration is a token, and the shipped
components rely on the token override. Recorded in [08-drift.md](08-drift.md).

Two consequences to respect:

- A hardcoded duration silently opts out of the contract. Always use a token.
- A `@keyframes` animation with a hardcoded duration also opts out. Keyframe work
  still needs an explicit reduced-motion branch, and `global.css:184-188` writes
  one for the draw-in idiom.

## Animatable properties

Allowed: `transform`, `opacity`, `stroke-dashoffset`, `filter` limited to `blur`
on the search overlay, and `clip-path` on staged reveals.

Banned: `width`, `height`, `top`, `left`, `margin`, `padding`, plus
`background-color` and `box-shadow` on large surfaces. Each of these triggers
layout or a paint storm.

`border-color` is animated on card hover. It is a paint on a 1px edge, so it is
cheap, and it is the site's primary hover signal.

## Acceleration

Animate `transform` and `opacity`. Promote sparingly with `will-change:
transform` and drop it once the transition ends. Never put `will-change` on a
scrolling container.

## Ambient motion budget

At most one perpetual animation per page. A single slow drift on a decorative
hero element is fine. Two or more makes the page restless and drains battery.

Scroll-triggered draw-in is not perpetual and does not count against this budget.
It runs once per element, on entry. See [06-idioms.md](06-idioms.md).

## Theme transition

`global.css:30` transitions `background-color` and `color` on `html, body` over
`--dur-med`:

```css
html, body { transition: background-color var(--dur-med) var(--ease-out), color var(--dur-med) var(--ease-out); }
```

This is the one deliberate exception to the ban on animating `background-color`
on a large surface. It fires only on an explicit theme toggle, never during
scroll or hover, so the repaint cost is paid once per user action. There is no
site-wide cross-fade beyond it; that would cost paint without earning anything.

## Focus

`global.css:40-44`:

```css
:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
  border-radius: var(--radius-sm);
}
```

The accent ring is the focus indicator. Do not remove it without an equivalent
replacement, and do not pair it with a `box-shadow` glow. `:focus-visible` rather
than `:focus` keeps the ring off mouse clicks and on keyboard traversal.
