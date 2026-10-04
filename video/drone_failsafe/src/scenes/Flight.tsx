// One fail-safe flies the night, full screen, in the still world. A running log lists each event at
// the second the fail-safe's trace records it, one line under the other.
import React from "react";
import { AbsoluteFill, interpolate, useCurrentFrame } from "remotion";
import { C, MONO, SANS } from "../tokens";
import { droneAt, flights, roundAt, type Part } from "../data";
import script from "../script.json";
import { WorldView } from "../views/WorldView";
import { clamp } from "./common";
import { STILL_WORLD } from "./world";

const FPS = 30;
type LogLine = { t: number; text: string; rule: string };

/** The verdicts of the latest round, on the ground for a few flight seconds after it. */
function verdictsAt(t: number) {
  const round = roundAt(t);
  if (!round || t - round.t > 5) return [];
  return round.rulings.map((r) => ({
    patch: r.patch,
    kind: (r.review === "Approved" ? "approved" : r.review === "Look" ? "look" : "forbidden") as "approved" | "look" | "forbidden",
  }));
}

type FlightConfig = {
  part: Part;
  title: string;
  /** Screen second -> flight second. The clock slows where events crowd together. */
  warp: [number, number][];
  log: LogLine[];
  outcome: string;
};

export const CONFIGS: Record<"textbook" | "ethos", FlightConfig> = {
  textbook: {
    part: 1,
    title: "TUTORIAL PART 1 · TEXTBOOK FAIL-SAFE",
    warp: [[0, 47], [2.5, 50.5], [5.5, 56.5], [8.5, 58.2], [12, 66], [14.5, 69.3], [18.5, 84], [20.5, 86], [22, 91.2]],
    log: script.textbookLog,
    outcome: script.textbookOutcome,
  },
  ethos: {
    part: 4,
    title: "TUTORIAL PART 4 · DYNAMIC FAIL-SAFE",
    warp: [[0, 47], [2.5, 50.5], [5.5, 56.5], [9.5, 58.3], [13.5, 64.5], [17, 69.5], [21, 77.3], [25.5, 85.3], [29.5, 90.5], [31, 92]],
    log: script.ethosLog,
    outcome: script.ethosOutcome,
  },
};
/** The screen second at which a flight's clock reaches flight second `flightT`. */
export const screenSecondAt = (which: "textbook" | "ethos", flightT: number) =>
  interpolate(flightT, CONFIGS[which].warp.map((w) => w[1]), CONFIGS[which].warp.map((w) => w[0]), clamp);

export const TEXTBOOK_FRAMES = 750;
export const ETHOS_FRAMES = 1020;

const STATUS: Record<string, string> = {
  Continue: "flying the line",
  Hold: "hold",
  LandNow: "land now",
  Routine: "flying the line",
  Practicable: "contingency",
  Possible: "emergency",
};

const FlightScene: React.FC<{ which: "textbook" | "ethos" }> = ({ which }) => {
  const cfg = CONFIGS[which];
  const frame = useCurrentFrame();
  const sec = frame / FPS;
  const t = interpolate(sec, cfg.warp.map((w) => w[0]), cfg.warp.map((w) => w[1]), clamp);
  const shownAt = (flightT: number) => screenSecondAt(which, flightT);
  const drone = droneAt(cfg.part, t);
  const ethosRun = cfg.part === 4;
  const lines = cfg.log.filter((l) => l.t <= t);
  const X = (s: number) => 96 + (1728 * s) / 100;
  const landedAt = shownAt(flights[cfg.part].touchdown.t);
  return (
    <AbsoluteFill style={{ backgroundColor: C.bg0 }}>
      <WorldView
        part={cfg.part}
        t={t}
        knows={ethosRun}
        wind={!ethosRun}
        track={!ethosRun}
        keepOut={ethosRun}
        verdicts={ethosRun ? verdictsAt(t) : []}
        target={ethosRun && drone.row.plan === "Approved" ? drone.row.target : null}
        {...STILL_WORLD}
      />
      <div style={{ position: "absolute", left: 96, top: 62, fontFamily: MONO, fontSize: 24, letterSpacing: 2, color: ethosRun ? C.accent : C.fg1 }}>{cfg.title}</div>
      <div style={{ position: "absolute", right: 96, top: 62, fontFamily: MONO, fontSize: 22, color: C.fg1 }}>
        {drone.landed ? "landed" : `${Math.round(drone.agl)} m · ${STATUS[drone.row.decision] ?? drone.row.decision}`}
      </div>
      <div style={{ position: "absolute", left: 96, top: 116, width: 820 }}>
        {lines.length > 0 && (
          <div style={{ padding: "16px 22px 10px", borderRadius: 10, background: C.bg1, border: `1px solid ${C.line1}`, opacity: 0.94 }}>
            {lines.map((l, k) => {
              const latest = k === lines.length - 1;
              const fade = interpolate(sec, [shownAt(l.t), shownAt(l.t) + 0.4], [0, 1], clamp);
              return (
                <div key={k} style={{ display: "flex", gap: 18, marginBottom: 8, opacity: fade * (latest ? 1 : 0.6) }}>
                  <span style={{ fontFamily: MONO, fontSize: 20, color: C.fg2, width: 52, flexShrink: 0, paddingTop: 6 }}>{`${l.t} s`}</span>
                  <div>
                    <div style={{ fontFamily: SANS, fontSize: 26, lineHeight: 1.25, color: C.fg0 }}>{l.text}</div>
                    {l.rule && <div style={{ fontFamily: MONO, fontSize: 18, color: C.accent, marginTop: 2 }}>{l.rule}</div>}
                  </div>
                </div>
              );
            })}
          </div>
        )}
        {drone.landed && (
          <div style={{ marginTop: 16, padding: "14px 22px", borderRadius: 10, background: C.bg1, border: `1px solid ${ethosRun ? C.accent : C.fg2}`, opacity: interpolate(sec, [landedAt - 0.4, landedAt], [0, 1], clamp) }}>
            <div style={{ fontFamily: SANS, fontSize: 40, color: ethosRun ? C.accent : C.fg0 }}>{cfg.outcome}</div>
          </div>
        )}
      </div>
      <svg width={1920} height={1080} style={{ position: "absolute", left: 0, top: 0 }}>
        <line x1={X(0)} y1={1036} x2={X(100)} y2={1036} stroke={C.line2} strokeWidth={3} />
        <line x1={X(0)} y1={1036} x2={X(t)} y2={1036} stroke={C.fg2} strokeWidth={3} />
        {[[55, "GPS lost 55 s"], [65, "link lost 65 s"], [85, "cell fails 85 s"]].map(([s, label]) => (
          <g key={s}>
            <line x1={X(s as number)} y1={1028} x2={X(s as number)} y2={1044} stroke={t >= (s as number) ? C.fg0 : C.fg2} strokeWidth={2} />
            <text x={X(s as number)} y={1066} fill={t >= (s as number) ? C.fg0 : C.fg2} fontFamily={MONO} fontSize={18} textAnchor="middle">{label}</text>
          </g>
        ))}
        <circle cx={X(t)} cy={1036} r={8} fill={C.accent} />
        <text x={X(t)} y={1016} fill={C.accent} fontFamily={MONO} fontSize={18} textAnchor="middle">{`${Math.floor(t)} s`}</text>
      </svg>
    </AbsoluteFill>
  );
};

export const TextbookFlight: React.FC = () => <FlightScene which="textbook" />;
export const EthosFlight: React.FC = () => <FlightScene which="ethos" />;
