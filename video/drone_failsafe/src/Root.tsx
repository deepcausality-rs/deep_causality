import React from "react";
import { Composition } from "remotion";
import { Explainer, totalFrames, type SceneName } from "./Explainer";
import { loadFonts } from "./fonts";

loadFonts();

const CUTS: { id: string; scenes: SceneName[] }[] = [
  { id: "Main", scenes: ["title", "setting", "part1", "textbook", "issue", "part2", "ethos", "simulations", "summary", "end"] },
  // A post for part 1: the textbook fail-safe and the issue with its static rule.
  { id: "Short-Part1", scenes: ["title", "setting", "part1", "textbook", "issue", "end"] },
  // A post for part 2: the dynamic fail-safe, the evidence and the three steps.
  { id: "Short-Part2", scenes: ["title", "part2", "ethos", "simulations", "summary", "end"] },
];

export const Root: React.FC = () => (
  <>
    {CUTS.map((cut) => (
      <Composition key={cut.id} id={cut.id} component={Explainer} durationInFrames={totalFrames(cut.scenes)} fps={30} width={1920} height={1080} defaultProps={{ scenes: cut.scenes }} />
    ))}
  </>
);
