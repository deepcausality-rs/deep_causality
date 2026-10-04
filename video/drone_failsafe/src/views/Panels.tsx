// The lower right panel: the faults and the fail-safe ladder in parts 1 to 3, the Effect Ethos in
// part 4.
import React from "react";
import { C, MONO, PANEL_W, PANEL_X, SANS } from "../tokens";
import { ethos, roundAt, type Row } from "../data";

const TOP = 532;
const HEIGHT = 380;

const Frame: React.FC<{ title: string; children: React.ReactNode }> = ({ title, children }) => (
  <g>
    <rect x={PANEL_X} y={TOP} width={PANEL_W} height={HEIGHT} rx={8} fill={C.bg1} stroke={C.line1} />
    <text x={PANEL_X + 22} y={TOP + 34} fill={C.fg2} fontFamily={MONO} fontSize={14} letterSpacing={1.4}>{title}</text>
    {children}
  </g>
);

const FAULTS = ["fix degraded", "fix lost", "link lost", "battery critical"];
const LADDER: [string, string][] = [
  ["Continue", "continue"],
  ["Hold", "hold"],
  ["ReturnHome", "return home"],
  ["LandNow", "land now"],
];

/** The confirmed faults and the step of the ladder in force. */
export const LadderPanel: React.FC<{ row: Row; extra?: string }> = ({ row, extra }) => (
  <Frame title="CONFIRMED FAULTS · FAIL-SAFE LADDER">
    {FAULTS.map((name, k) => (
      <g key={name}>
        <circle cx={PANEL_X + 32} cy={TOP + 72 + k * 34} r={6} fill={row.faults[k] ? C.fg0 : "none"} stroke={row.faults[k] ? C.fg0 : C.line2} strokeWidth={1.6} />
        <text x={PANEL_X + 50} y={TOP + 78 + k * 34} fill={row.faults[k] ? C.fg0 : C.fg2} fontFamily={MONO} fontSize={17}>{name}</text>
      </g>
    ))}
    {LADDER.map(([key, label], k) => {
      const on = row.decision === key;
      return (
        <g key={key}>
          <rect x={PANEL_X + 300} y={TOP + 56 + (3 - k) * 40} width={226} height={32} rx={5} fill={on ? C.accent : C.bg0} stroke={on ? C.accent : C.line2} />
          <text x={PANEL_X + 413} y={TOP + 78 + (3 - k) * 40} fill={on ? "#062029" : C.fg2} fontFamily={MONO} fontSize={16} textAnchor="middle">{label}</text>
        </g>
      );
    })}
    {extra && (
      <text x={PANEL_X + 22} y={TOP + 260} fill={C.fg1} fontFamily={MONO} fontSize={16}>{extra}</text>
    )}
  </Frame>
);

const URGENCY: Record<string, string> = {
  Routine: "routine",
  Practicable: "contingency",
  Possible: "emergency",
  LastResort: "last resort",
};
export const urgencyName = (u: string) => URGENCY[u] ?? u;

/** The latest Effect Ethos round, and the last-resort prices once they are shown. */
export const EthosPanel: React.FC<{ t: number; prices: number }> = ({ t, prices }) => {
  const round = roundAt(t);
  const permitted = round ? round.rulings.filter((r) => r.review === "Approved").length : 0;
  const reasons = round
    ? round.rulings.filter((r) => r.review !== "Approved").map((r) => r.norms.join(";"))
    : [];
  const common = reasons.length ? [...reasons].sort((a, b) => reasons.filter((r) => r === b).length - reasons.filter((r) => r === a).length)[0] : "";
  const NORM: Record<string, string> = {
    "2": "person within 13 m",
    "3": "person not ruled out",
    "4": "battery falls short",
    "5": "touchdown not survivable",
    "8": "path crosses trees or unseen ground",
    "14": "a gust could tip it",
  };
  const why = common
    .split(";")
    .filter(Boolean)
    .map((n) => NORM[n] ?? `norm ${n}`)
    .join(" · ");
  return (
    <Frame title={`EFFECT ETHOS · ${round ? urgencyName(round.rulings[0].urgency).toUpperCase() + " ROUND" : "NO ROUND YET"}`}>
      {round ? (
        <>
          <text x={PANEL_X + 22} y={TOP + 74} fill={C.fg0} fontFamily={MONO} fontSize={18}>
            {`t ${round.t} s · ${round.rulings.length} proposals · ${permitted} permitted`}
          </text>
          {why && (
            <>
              <text x={PANEL_X + 22} y={TOP + 102} fill={C.fg2} fontFamily={MONO} fontSize={13.5}>most often forbidden by</text>
              <text x={PANEL_X + 22} y={TOP + 122} fill={C.fg1} fontFamily={MONO} fontSize={12.5}>{why}</text>
            </>
          )}
          <text x={PANEL_X + 22} y={TOP + 154} fill={permitted ? C.accent : C.fg1} fontFamily={MONO} fontSize={17}>
            {permitted ? "the drone flies the cheapest permitted patch" : "nothing permitted yet: hold and look"}
          </text>
        </>
      ) : (
        <text x={PANEL_X + 22} y={TOP + 74} fill={C.fg2} fontFamily={MONO} fontSize={16}>the drone flies its mission</text>
      )}
      <line x1={PANEL_X + 22} y1={TOP + 170} x2={PANEL_X + PANEL_W - 22} y2={TOP + 170} stroke={C.line1} />
      <text x={PANEL_X + 22} y={TOP + 202} fill={C.fg2} fontFamily={MONO} fontSize={14} letterSpacing={1.4} opacity={prices}>
        LAST RESORT · COST OF EACH DITCHING
      </text>
      {[0, 2, 1].map((n, k) => {
        const r = ethos.lastResort[n];
        const [letter, label, danger] = [["a", "beside the crew", true], ["c", "on empty steep grass", false], ["b", "in the creek", false]][n] as [string, string, boolean];
        const col = danger ? C.danger : C.fg0;
        const y = TOP + 244 + k * 38;
        return (
          <g key={letter} opacity={prices}>
            <circle cx={PANEL_X + 32} cy={y - 6} r={11} fill={C.bg0} stroke={col} strokeWidth={1.6} />
            <text x={PANEL_X + 32} y={y - 1.5} fill={col} fontFamily={MONO} fontSize={13} fontWeight={600} textAnchor="middle">{letter}</text>
            <text x={PANEL_X + 54} y={y} fill={C.fg1} fontFamily={SANS} fontSize={18}>{label}</text>
            <text x={PANEL_X + PANEL_W - 22} y={y} fill={col} fontFamily={MONO} fontSize={19} textAnchor="end">{r.cost.toLocaleString("en-US")}</text>
          </g>
        );
      })}
    </Frame>
  );
};
