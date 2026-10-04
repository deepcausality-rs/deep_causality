import React from "react";
import { Composition } from "remotion";
import { CutVideo } from "./Video";
import { timeline } from "./data";
import { loadFonts } from "./fonts";

loadFonts();

const CLIP_NAMES = ["Teaser", "Run1", "Run2", "Run3", "Twist", "Run4", "Campaign"];

export const Root: React.FC = () => (
  <>
    <Composition id="Main" component={CutVideo} durationInFrames={timeline.main.durationInFrames} fps={timeline.fps} width={1920} height={1080} defaultProps={{ cut: timeline.main, captions: false }} />
    {timeline.clips.map((clip, k) => (
      <Composition key={k} id={`Clip${k}-${CLIP_NAMES[k]}`} component={CutVideo} durationInFrames={clip.durationInFrames} fps={timeline.fps} width={1920} height={1080} defaultProps={{ cut: clip, captions: true }} />
    ))}
  </>
);
