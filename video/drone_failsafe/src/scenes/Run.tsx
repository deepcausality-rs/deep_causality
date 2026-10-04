// One flight of the default night, parts 1 to 4: the world in 3D on the left, what the drone knows
// and decides in 2D on the right. Flight time follows the narration: each anchor pins the flight's
// second at the start of one sentence.
import React from "react";
import { AbsoluteFill, interpolate, useCurrentFrame } from "remotion";
import { C } from "../tokens";
import { droneAt, ethos, type Part, type SceneSpec } from "../data";
import { WorldView } from "../views/WorldView";
import { NoGroundView, TopView } from "../views/TopView";
import { EthosPanel, LadderPanel, urgencyName } from "../views/Panels";
import { Chips, Eyebrow, Frame1080, OutcomeCard, Timeline } from "../views/Hud";

type RunConfig = {
  part: Part;
  title: string;
  start: number;
  /** [sentence index, flight second at its start] */
  anchors: [number, number][];
  end: number;
  outcome: string[];
  danger?: boolean;
};

export const RUNS: Record<string, RunConfig> = {
  run1: { part: 1, title: "PART 1 · DYNAMIC CAUSALITY", start: 40, anchors: [[1, 52], [2, 68], [3, 72]], end: 91, outcome: ["In the creek.", "Every decision was correct."] },
  run2: { part: 2, title: "PART 2 · DYNAMIC CONTEXT", start: 34, anchors: [[1, 44], [2, 70], [3, 84], [4, 88], [5, 89.5]], end: 91, outcome: ["In the creek.", "It knew the ground."] },
  run3: { part: 3, title: "PART 3 · DYNAMIC ACTION", start: 46, anchors: [[1, 56], [2, 76], [3, 86]], end: 92, outcome: ["3 m from a person.", "The person was on its own map."], danger: true },
  run4: { part: 4, title: "PART 4 · EFFECT ETHOS", start: 46, anchors: [[1, 50], [2, 55], [3, 57.3], [4, 59.5], [5, 63.8], [6, 66], [7, 84.6], [8, 87.5], [9, 90]], end: 90, outcome: ["Upright, 34 m from the nearest person.", "The drone can be recovered."] },
};

const MANEUVER: Record<string, string> = {
  None: "none",
  HoldOver: "hold over the ground",
  ReturnHome: "fly home",
  LandOn: "land on the patch",
  ChooseTarget: "choose a patch",
  LookOver: "fly over the patch and look",
  Explore: "explore unseen ground",
};

/** The flight second at a scene frame, and how far the scene is into its final hold. */
export function flightTime(cfg: RunConfig, spec: SceneSpec, frame: number): { t: number; hold: number } {
  const last = spec.cues[spec.cues.length - 1];
  const activeEnd = last.from + last.durationInFrames;
  const points: [number, number][] = [[0, cfg.start], ...cfg.anchors.map(([k, t]) => [spec.cues[k].from, t] as [number, number]), [activeEnd, cfg.end]];
  const t = interpolate(frame, points.map((p) => p[0]), points.map((p) => p[1]), { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const hold = interpolate(frame, [activeEnd, activeEnd + 12], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  return { t, hold };
}

const CREW_NEAR: [number, number] = [58, 449];

export const Run: React.FC<{ spec: SceneSpec; name: string }> = ({ spec, name }) => {
  const frame = useCurrentFrame();
  const cfg = RUNS[name];
  const { t, hold } = flightTime(cfg, spec, frame);
  const part = cfg.part;
  const drone = droneAt(part, t);
  const row = drone.row;
  const cue = (k: number) => spec.cues[k];
  const within = (k: number, a = 0, b = 1) => frame >= cue(k).from + cue(k).durationInFrames * a && frame <= cue(k).from + cue(k).durationInFrames * b;
  const dayRule = part === 2 && within(3, 0.45, 1);
  const prices = part === 4 ? interpolate(frame, [cue(9).from, cue(9).from + 15], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" }) : 0;
  const pins =
    prices > 0
      ? [
          { patch: ethos.lastResort[0].patch, letter: "a", danger: true },
          { patch: ethos.lastResort[2].patch, letter: "b", danger: false },
          { patch: ethos.lastResort[1].patch, letter: "c", danger: false },
        ]
      : [];
  const target = part >= 3 ? row.target : null;
  const landed = drone.landed;
  const link = landed && part === 3 ? { to: CREW_NEAR, label: "3 m", danger: true } : landed && part === 4 ? { to: CREW_NEAR, label: "34 m", danger: false } : null;
  const decision = part === 4 ? urgencyName(row.decision) : row.decision.replace(/([a-z])([A-Z])/g, "$1 $2").toLowerCase();
  const chips: [string, string, boolean?][] = [
    ["t", `${Math.floor(t)} s`],
    ["height", `${Math.round(drone.agl)} m`],
    [part === 4 ? "urgency" : "fail-safe", landed ? "landed" : decision, row.decision === "Possible" || row.decision === "LandNow"],
  ];
  if (part >= 3 && !landed) chips.push(["maneuver", MANEUVER[row.maneuver] ?? row.maneuver]);
  return (
    <AbsoluteFill style={{ backgroundColor: C.bg0 }}>
      <WorldView part={part} t={t} knows={part >= 2} dayRule={dayRule} wind={part <= 2} track={part <= 2} keepOut={part === 4} target={target} pins={pins} link={link} />
      <Frame1080>
        <rect x={0} y={0} width={1266} height={150} fill={C.bg0} opacity={0.82} />
        <rect x={1266} y={0} width={654} height={1080} fill={C.bg0} />
        <Eyebrow text={`DEEPCAUSALITY · ${cfg.title}`} />
        <Chips items={chips} />
        {part === 1 ? <NoGroundView /> : <TopView part={part} t={t} dayRule={dayRule} target={target} showRound={part === 4} pins={pins} />}
        {part === 4 ? <EthosPanel t={t} prices={prices} /> : <LadderPanel row={row} extra={part === 3 && row.maneuver !== "None" ? `fail-safe machine: ${MANEUVER[row.maneuver] ?? row.maneuver}` : undefined} />}
        <Timeline part={part} t={t} />
        <OutcomeCard lines={cfg.outcome} danger={cfg.danger} opacity={hold} />
      </Frame1080>
    </AbsoluteFill>
  );
};
