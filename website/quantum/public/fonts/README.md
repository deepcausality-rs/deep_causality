# Vendored fonts

These woff2 files are served directly from the same origin as the rest of the site. No CDN, no node_modules indirection at runtime, no render-blocking external request.

| File | Family | Subset | Source |
|---|---|---|---|
| `geist-latin.woff2` | Geist Variable | Latin | `@fontsource-variable/geist@5.2.8` |
| `geist-latin-ext.woff2` | Geist Variable | Latin Extended | `@fontsource-variable/geist@5.2.8` |
| `jetbrains-mono-latin.woff2` | JetBrains Mono Variable | Latin | `@fontsource-variable/jetbrains-mono@5.2.8` |
| `jetbrains-mono-latin-ext.woff2` | JetBrains Mono Variable | Latin Extended | `@fontsource-variable/jetbrains-mono@5.2.8` |

To refresh: `pnpm add -D @fontsource-variable/geist @fontsource-variable/jetbrains-mono`, copy the `latin-wght-normal.woff2` and `latin-ext-wght-normal.woff2` files into this directory under the renamed paths above, then `pnpm remove` the packages.

The `@font-face` declarations that bind these files to the CSS font families live in `src/styles/fonts.css`.
