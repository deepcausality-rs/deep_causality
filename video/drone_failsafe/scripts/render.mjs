// Bundles the project once and renders from that bundle.
import { bundle } from "@remotion/bundler";
import { renderMedia, renderStill, selectComposition, getCompositions } from "@remotion/renderer";
import { mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import os from "node:os";

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const [mode, ...args] = process.argv.slice(2);
const valid =
  (mode === "stills" && args.length >= 2 && args.slice(1).every((f) => /^\d+$/.test(f))) ||
  (mode === "video" && args.length >= 1) ||
  (mode === "all" && args.length === 0);
if (!valid) {
  console.error(`usage:
  node scripts/render.mjs stills <composition> <frame> [<frame> ...]   PNG stills into out/stills/
  node scripts/render.mjs video <composition> [<composition> ...]      MP4s into out/
  node scripts/render.mjs all                                          every composition`);
  process.exit(1);
}
const serveUrl = await bundle({ entryPoint: join(root, "src", "index.ts"), publicDir: join(root, "public") });
mkdirSync(join(root, "out", "stills"), { recursive: true });
const chromium = { gl: "angle" };

if (mode === "stills") {
  const [id, ...frames] = args;
  const composition = await selectComposition({ serveUrl, id, chromiumOptions: chromium });
  for (const frame of frames.map(Number)) {
    const output = join(root, "out", "stills", `${id}_${frame}.png`);
    await renderStill({ serveUrl, composition, frame, output, chromiumOptions: chromium });
    console.log(output);
  }
} else {
  const ids = mode === "all" ? (await getCompositions(serveUrl)).map((c) => c.id) : args;
  for (const id of ids) {
    const composition = await selectComposition({ serveUrl, id, chromiumOptions: chromium });
    const output = join(root, "out", `${id}.mp4`);
    const started = Date.now();
    let last = -1;
    await renderMedia({
      serveUrl,
      composition,
      codec: "h264",
      crf: 18,
      imageFormat: "jpeg",
      jpegQuality: 92,
      concurrency: Math.max(1, Math.floor(os.cpus().length * 0.75)),
      chromiumOptions: chromium,
      outputLocation: output,
      onProgress: ({ progress }) => {
        const pct = Math.floor(progress * 10);
        if (pct !== last) {
          last = pct;
          console.log(`${id}: ${pct * 10} %`);
        }
      },
    });
    console.log(`${output} in ${Math.round((Date.now() - started) / 1000)} s`);
  }
}
