/// <reference lib="dom.iterable" />
// dom.iterable declares FontFaceSet as a Set<FontFace>, which is where `document.fonts.add` is typed.
import { cancelRender, continueRender, delayRender, staticFile } from "remotion";

const faces: [string, string][] = [
  ["Geist", "fonts/geist-latin.woff2"],
  ["JetBrains Mono", "fonts/jetbrains-mono-latin.woff2"],
];

let loaded: Promise<void> | null = null;

/** Loads Geist and JetBrains Mono once; rendering waits until both are ready. A failed load cancels the render. */
export function loadFonts(): void {
  if (loaded) return;
  const handle = delayRender("fonts");
  loaded = Promise.all(
    faces.map(async ([family, file]) => {
      const face = new FontFace(family, `url(${staticFile(file)}) format("woff2")`, { weight: "100 900" });
      await face.load();
      document.fonts.add(face);
    }),
  )
    .then(() => continueRender(handle))
    .catch((err: unknown) => {
      loaded = null;
      cancelRender(err);
    });
}
