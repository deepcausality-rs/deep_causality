import React from "react";
import { Composition } from "remotion";
import { CUTS, Explainer, totalFrames } from "./Explainer";
import { loadFonts } from "./fonts";

loadFonts();

export const Root: React.FC = () => (
  <>
    {CUTS.map((cut) => (
      <Composition key={cut.id} id={cut.id} component={Explainer} durationInFrames={totalFrames(cut.scenes)} fps={30} width={1920} height={1080} defaultProps={{ scenes: cut.scenes }} />
    ))}
  </>
);
