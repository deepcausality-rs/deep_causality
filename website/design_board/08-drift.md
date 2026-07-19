# 08 — Drift

Where the shipped site and `website/web/DESIGN.md` disagree, and where the site
disagrees with itself. Ranked by consequence, not by count.

## Status

DESIGN.md was reconciled against the implementation on 2026-07-20. Every entry
below is now tagged:

| Tag | Meaning |
|---|---|
| **SPEC RECONCILED** | DESIGN.md updated to match reality. No code change pending. |
| **SPEC RECORDS, CODE OPEN** | DESIGN.md now documents the defect and names the fix. Code unchanged. |
| **OPEN** | Needs a decision before anything can be written down as settled. |

Nothing here is a change request. Each entry records a fact and names the
options.

---

## 1. Every contrast ratio in DESIGN.md is wrong, and two are real failures

> **SPEC RECORDS, CODE OPEN** — DESIGN.md §2 and §2.1 now carry measured ratios and flag both AA failures; §10 no longer claims AA is met. The light-accent colour decision is still open.

`DESIGN.md` §2 and §2.1 publish a contrast column. Recomputed from the shipped
hex values under WCAG 2.1, not one of the fourteen rows matches. Twelve are
conservative, understating the true ratio by up to 2.3. Three are optimistic, and
two of those cross a compliance threshold.

| Token | Theme | Claimed | Actual | Consequence |
|---|---|---|---|---|
| `--accent` | light | 4.9 AA | **4.12** | Every link in light mode fails AA |
| `--accent-ink` on `--accent` | light | 4.9 AA | **4.12** | Primary CTA text fails AA |
| `--fg-2` | dark | 4.6 AA | **4.26** | Fails AA on the page background |

`--fg-2` in dark mode degrades further on raised surfaces: 4.10 on `--bg-1`, 3.79
on `--bg-2`, 3.39 on `--bg-3`.

This contradicts `DESIGN.md` §10, which claims AA everywhere and AAA on body
prose.

Options:

- **Darken the light accent.** `#0a8a98` → roughly `#087b88` reaches 4.5 on
  white. It also darkens every accent tick and node dot in light mode.
- **Keep `#0a8a98` for large text and non-text UI only**, and give body links a
  separate darker token. More tokens, no visual change to the accent itself.
- **Restrict `--fg-2` to non-text use** and move meta text to `--fg-1`. Cheapest
  of the three and matches how `--fg-2` is already used for ticks and rules.

Correct measurements are in [01-foundations.md](01-foundations.md).

---

## 2. Four home components hardcode their frame width

> **SPEC RECORDS, CODE OPEN** — DESIGN.md §12.7 names `--w-panel` as the fix.

`--w-prose`, `--w-doc`, `--w-page`, and `--w-wide` exist. These four ignore them:

| Component | Value |
|---|---|
| `JoinCommunity.astro:62` | `max-width: 880px` |
| `CausalStack.astro:101` | `max-width: 900px` |
| `Explainer.astro:59` | `max-width: 980px` |
| `ExampleGrid.astro:158` | `max-width: 980px` |

Three different widths stacked vertically on one page. The left and right edges
of the landing page do not line up between sections, which reads as sloppiness
rather than as rhythm.

Options: introduce a `--w-panel` token at one agreed value and point all four at
it, or accept the variance and document it as intentional. It is currently
neither.

---

## 3. `--measure` is bypassed almost everywhere

> **SPEC RECORDS, CODE OPEN** — DESIGN.md §12.7.

`--measure: 68ch` is used by `.static-page` and nothing else. Components write
literal `ch` values instead: `44ch`, `52ch` (×2), `56ch` (×3), `60ch` (×5).

Five distinct measures for one typographic decision. Consolidating to two — a
prose measure and a shorter lede measure — would cover every current use.

---

## 4. One gesture, three treatments

> **SPEC RECONCILED** — DESIGN.md §12.11 declares the canonical lift and press.

The hover lift is written three ways:

| Component | Transform | Duration |
|---|---|---|
| `ExampleCard.astro:60,63` | `translate3d(0, -2px, 0)` | `--dur-med` |
| `JoinCommunity.astro:88,99` | `translateY(-1px)` | `--dur-fast` |
| `Hero.astro:87` | `translateY(1px)` (press) | `--dur-fast` |

Distance, direction, and duration all vary. `translate3d` also forces GPU
promotion while `translateY` may not, so the two do not even composite alike.

Pick one lift (`translate3d(0, -2px, 0)` at `--dur-med`) and one press
(`translateY(1px)` at `--dur-fast`).

Related bug: `ExampleCard.astro:65` animates `box-shadow` on hover, but `:60`
omits `box-shadow` from the transition list. The halo snaps rather than fading.
The component is currently orphaned, so nothing renders the bug.

---

## 5. `.eyebrow` is redeclared in nine files

> **SPEC RECONCILED** — DESIGN.md §12.1 declares the global class canonical.

`global.css:104` defines it. Hero, WhyDeepCausality, ExampleGrid, ExampleDetail,
CategoryList, `blog/index`, `blog/[...slug]`, `examples/index`, and `404` each
redeclare it locally. Only three re-apply `font-mono`, so the remaining six lose
the mono treatment that makes the idiom work.

`404.astro:44` is the correct version. `SiteHeader.astro:326` sets the eyebrow
size without the mono family and is the clearest instance of the bug.

Worse, the tick itself has three implementations at two sizes: `12px` in
`global.css:112`, `10px` in `ExampleCard.astro:84` and `blog/index.astro:282`,
and an absolutely-positioned `8px` variant in `Explainer.astro:118-127`. The flex
gap is `0.55em` in two places and `0.5em` in a third.

Deleting the nine local redeclarations is a mechanical fix and would make the
global rule authoritative.

---

## 6. The L-bracket motif has three implementations

> **SPEC RECONCILED** — DESIGN.md §12.5 declares the 10px pseudo-element form canonical.

Same visual, three code paths, two sizes:

- `ExampleGrid.astro:330-347` — pseudo-elements, 10×10, two corners
- `overview/index.astro:407-418` — pseudo-elements, 10×10, two corners
- `ExampleDetail.astro:202-221` — SVG data-URI masks, 14×14, four corners

The first two share an identical colour expression,
`color-mix(in srgb, var(--accent) 60%, var(--line-2))`, so they were clearly
copied. A `.corner-brackets` utility in `global.css`, alongside `.reticle`, would
collapse all three.

---

## 7. The 80px header offset does not match the header

> **SPEC RECORDS, CODE OPEN** — DESIGN.md §9.4 and §12.7 name `--header-h` as the fix. Anchor links still land 16px low.

`blog/index.astro` uses `80px` four times (`:172`, `:180`, `:219`, `:222`) for
`scroll-margin-top` and sticky offsets. The header is `min-height: 56px` base and
`64px` at ≥900px (`SiteHeader.astro:141,150`).

Nothing on the site is 80px tall. Anchor links currently land 16px lower than
intended. A `--header-h` token consumed by both the header and its offsets would
fix the value and the duplication together.

The same file duplicates the 900px breakpoint as a JS string (`:123`).

---

## 8. Glassmorphism is banned, and shipped

> **SPEC RECONCILED** — DESIGN.md §13 item 9 now sanctions the scrim exception explicitly.

`DESIGN.md` §12 item 9 bans `backdrop-filter: blur(...)`. `SiteHeader.astro:298-299`
applies `blur(2px)` to the mobile menu scrim.

This is defensible. The ban targets frosted panels over the hero and the docs
sidebar; a 2px blur on a dismissible scrim is a different thing. But the rule as
written is absolute, so either the rule gains an exception for scrims or the blur
goes.

---

## 9. The reduced-motion contract is inverted, and the code is right

> **SPEC RECONCILED** — DESIGN.md §6 rewritten to specify the token-level contract.

`DESIGN.md` §6 requires every transition to sit inside
`@media (prefers-reduced-motion: no-preference)`. The shipped system instead
zeroes the duration tokens under `prefers-reduced-motion: reduce`
(`tokens.css:135-141`), which makes any token-based transition compliant with no
wrapper.

The implementation is better than the spec. Update `DESIGN.md`, not the code.

The caveat belongs in the spec too: hardcoded durations silently opt out. Four
currently exist — `110ms` and `280ms` in `CausalStack.astro:243-244`, and `220`
and `150` in JS at `SiteHeader.astro:110` and `ExampleGrid.astro:128`. The
SiteHeader value must track `--dur-med` by hand.

---

## 10. Components DESIGN.md specifies that do not exist

> **SPEC RECONCILED**, except search. DESIGN.md §7.1, §8.3, §8.4, §8.6, §9.2 and §9.5 now match the built site. Search remains **OPEN**: §8.9 records that Pagefind indexes on every build and nothing consumes it.

`DESIGN.md` §7.1 lists a file layout largely unbuilt. Absent: `DocsLayout.astro`,
`ProseLayout.astro`, `MobileMenu.astro` (folded into SiteHeader), `Callout.astro`,
`CodeBlock.astro`, `Toc.astro`, `Sidebar.astro`, `PillarRow.astro`, the whole
`ui/` directory, and the `prose.css` / `code.css` stylesheets.

Two are structural, not cosmetic:

**Search does not exist.** §8.8 specifies a Pagefind trigger with ⌘K and a modal.
There is no search component, and §7.3 calls `SearchTrigger` the site's one
`client:idle` island. There are zero client islands.

**The pillar row does not exist.** §8.4 specifies the Causaloid / Context /
Effect Ethos row connected by an SVG path. `CausalStack.astro` occupies that slot
with a different five-band concept.

Docs moved to a separate Starlight site at `website/docs/`, which explains the
missing docs components. Search and the pillar row were dropped or deferred
without the spec recording it.

---

## 11. The docs site hand-copies the palette

> **SPEC RECONCILED** — DESIGN.md §9.2 and §14 record the coupling and require same-commit updates.

`website/docs/src/styles/theme.css` maps this design system onto Starlight by
duplicating literal hex values, with the source token named in a trailing
comment:

```css
--sl-color-gray-2: #aab3bd; /* fg-1 */
--sl-color-black: #070b10;  /* bg-0 */
```

Twenty-six values across both themes. Changing `tokens.css` does not change them,
and nothing fails when they diverge. The comments make the intent auditable but
do not enforce it.

The docs theme also introduces four accent shades that do not exist in the main
token set: `#0e3a40`, `#b3ecf2` (dark), `#d7f0f3`, `#063e45` (light). If accent
tints are needed, they belong in `tokens.css` where both sites can read them.

---

## 12. Smaller items

> **SPEC RECONCILED** — the conventions are settled in DESIGN.md §12; the code cleanups remain open.

- **Five breakpoints for a two-breakpoint system.** 600px exists only in the
  orphaned `Explainer.astro`; 480px only in `blog/index.astro`; 1024px only in
  the `BaseLayout` footer.
- **Hardcoded font sizes.** `11px` (`ExampleGrid.astro:227`) and `14px`
  (`ExampleGrid.astro:321`). The second should be `--t-mono-block`, which the
  otherwise-identical `ExampleCard.astro:103` does use.
- **`font-weight: 540` appears literally six times** with no token, despite being
  the deliberate heading weight. Add `--fw-heading`.
- **Focus states are inconsistent.** `outline-offset` is `2px`, `-3px`, `4px`, or
  absent. `.btn`, `.chip`, `.community-card`, and `.theme-toggle` have no
  `:focus-visible` rule and fall back to the global ring. The `-3px` inset in
  ExampleGrid is justified by its clipped container; the others are not.
- **`<style is:global>` in `blog/[...slug].astro:105`** leaks `.post` rules
  site-wide from a route component.
- **Four `!important` declarations**: `ExampleGrid.astro:349`,
  `blog/index.astro:266`, `SiteHeader.astro:377`, `global.css:187`. The last is
  justified (a reduced-motion override); the others paper over specificity.
- **`blog/index.astro:331` references `--t-body-xs`**, a token that does not
  exist, with `--t-body-sm` as fallback. Harmless, but it reads as a leftover.
- **`.dc-diagram` uses `--radius-sm`** where every other framed panel uses
  `--radius-md` (`overview/index.astro:402`).
- **Dead `.eyebrow` CSS** in `Hero.astro:54-58`, `ExampleGrid.astro:146-152`, and
  `WhyDeepCausality.astro:125-135`, none of which render an `.eyebrow` element.

---

## Suggested order

Spec reconciliation is done. What remains is code work, in this order:

1. **§1 contrast** — an accessibility failure on shipped pages. Needs a colour
   decision before code.
2. **§7 header offset** — anchor links land 16px low. Add `--header-h`.
3. **§10 search** — decide whether Pagefind gets finished or removed. Every
   deploy currently ships an unused index.
4. **§5 eyebrow, §12 focus ring** — pure deletions of local overrides. No
   intended pixel changes, and they close the largest divergence surface.
5. **§2 frame widths, §3 measure, §6 brackets, §4 lift** — consolidation behind
   new tokens and shared utilities.
6. **§7.1 orphans** — decide on `ExampleCard` and `Explainer` before their
   vocabulary rots further.

Items 4 and 5 are mechanical and low-risk. Items 1, 3, and 6 need a decision
first and should not be started without one.
