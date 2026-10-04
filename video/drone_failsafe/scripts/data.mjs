// Reads the tutorial's traces and campaign record, checks every number the video shows against the
// tutorial README, and writes the JSON the compositions import. A mismatch stops the build.
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const traces = join(root, "public", "traces");
const tutorial = join(root, "..", "..", "end_to_end_tutorials", "dynamic_drone_failsafe");
const out = join(root, "src", "data");

const csv = (path) => {
  const [head, ...lines] = readFileSync(path, "utf8").trim().split("\n");
  const cols = head.split(",");
  return lines.map((line) => {
    const cells = line.split(",");
    return Object.fromEntries(cols.map((c, k) => [c, cells[k] ?? ""]));
  });
};
const num = (v) => (v === "" ? null : Number(v));
const bool = (v) => v === "true";
const failures = [];
const check = (label, actual, expected) => {
  const ok = typeof expected === "number" ? Math.abs(actual - expected) < 1e-9 : actual === expected;
  if (!ok) failures.push(`${label}: trace gives ${actual}, the README says ${expected}`);
};
const readme = readFileSync(join(tutorial, "README.md"), "utf8");
const inReadme = (label, text) => {
  if (!readme.includes(text)) failures.push(`${label}: the README holds no "${text}"`);
};

// The world on a 2 m grid.
const SURFACES = ["Water", "Grass", "Road", "Rock", "Pad", "Ravine", "Trees"];
const world = csv(join(traces, "world.csv"));
const xs = [...new Set(world.map((r) => Number(r.x)))];
const ys = [...new Set(world.map((r) => Number(r.y)))];
const worldJson = {
  x0: xs[0],
  y0: ys[0],
  step: xs[1] - xs[0],
  nx: xs.length,
  ny: ys.length,
  elevation: world.map((r) => Number(r.elevation_m)),
  surface: world.map((r) => SURFACES.indexOf(r.surface)),
  canopy: world.map((r) => Number(r.canopy_m)),
};
const crew = csv(join(traces, "crew.csv")).map((r) => [Number(r.x), Number(r.y)]);

// The four flights.
const GROUNDS = ["Safe", "Steep", "Water", "Person", "Unsure", "Trees"];
const flight = (n) =>
  csv(join(traces, `part_${n}_trace.csv`)).map((r) => ({
    t: Number(r.t),
    x: Number(r.x),
    y: Number(r.y),
    agl: Number(r.agl),
    faults: [bool(r.gnss_degraded), bool(r.fix_lost), bool(r.link_lost), bool(r.battery_critical)],
    decision: r.decision,
    below: r.below ?? "",
    maneuver: r.maneuver ?? "",
    target: r.target_i ? [Number(r.target_i), Number(r.target_j)] : null,
    plan: r.plan_status ?? "",
  }));
const touchdown = (n) => {
  const [r] = csv(join(traces, `part_${n}_touchdown.csv`));
  return { t: Number(r.t), x: Number(r.x), y: Number(r.y), outcome: r.outcome, surface: r.surface, nearestPersonM: Number(r.nearest_person_m) };
};
const patches = (n) =>
  csv(join(traces, `part_${n}_patches.csv`)).map((r) => [
    Number(r.t),
    Number(r.i),
    Number(r.j),
    GROUNDS.indexOf(r.ground),
    Math.round(Number(r.slope_sigma_deg) * 1000) / 1000,
  ]);
const flights = {};
for (const n of [1, 2, 3, 4]) flights[n] = { rows: flight(n), touchdown: touchdown(n) };
const patchJson = { 2: patches(2), 3: patches(3), 4: patches(4) };
for (const n of [2, 3, 4]) {
  if (patchJson[n].some((p) => p[3] < 0)) failures.push(`part ${n}: a patch judgement outside ${GROUNDS}`);
}

// The Effect Ethos rounds of part 4 and the last-resort rulings at the emergency.
const rulings = csv(join(traces, "part_4_rulings.csv")).map((r) => ({
  t: Number(r.t),
  urgency: r.urgency,
  patch: [Number(r.i), Number(r.j)],
  proposal: r.proposal,
  review: r.review,
  cost: num(r.harm_cost),
  norms: r.norms ? r.norms.split(";").map(Number) : [],
}));
const lastResort = csv(join(traces, "part_4_last_resort.csv")).map((r) => ({
  t: Number(r.t),
  kind: r.kind,
  patch: [Number(r.i), Number(r.j)],
  cost: Number(r.harm_cost),
}));

// The campaign record of part 5.
const ENDINGS = ["Resumed", "Safe", "NearPerson", "Ditched", "IntoRavine", "TippedAndRolled", "HitTrees", "Fell"];
const campaign = csv(join(tutorial, "part_5_verification", "campaign_1000.csv")).map((r) => [
  ENDINGS.indexOf(r.part_1_ending),
  ENDINGS.indexOf(r.part_3_ending),
  ENDINGS.indexOf(r.part_4_ending),
  r.cell_failed_s === "" ? 0 : 1,
]);

// One-sided Clopper-Pearson upper bound at 95 %, as part 5 computes it.
const upperBound = (k, n) => {
  const atMost = (p) => {
    let lnChoose = 0;
    let total = 0;
    for (let i = 0; i <= k; i++) {
      if (i > 0) lnChoose += Math.log(n - i + 1) - Math.log(i);
      total += Math.exp(lnChoose + i * Math.log(p) + (n - i) * Math.log(1 - p));
    }
    return total;
  };
  let [lo, hi] = [k / n, 1];
  for (let s = 0; s < 60; s++) {
    const mid = (lo + hi) / 2;
    if (atMost(mid) > 0.05) lo = mid;
    else hi = mid;
  }
  return hi;
};

// The numbers on screen, each checked against the README.
const at = (n, t) => flights[n].rows.find((r) => r.t === t);
const centre = ([i, j]) => [(i + 0.5) * 4, (j + 0.5) * 4];
const dist = (a, b) => Math.hypot(a[0] - b[0], a[1] - b[1]);
const p1 = flights[1].touchdown;
const p2day = csv(join(traces, "part_2_daytime.csv"))[0];
const p3 = flights[3].touchdown;
const p3choice = flights[3].rows.find((r) => r.target);
const p4 = flights[4].touchdown;
const rounds = Object.values(
  rulings.reduce((acc, r) => {
    const key = `${r.t}/${r.urgency}`;
    acc[key] ??= { t: r.t, urgency: r.urgency, proposals: 0, permitted: 0 };
    acc[key].proposals += 1;
    if (r.review === "Approved") acc[key].permitted += 1;
    return acc;
  }, {}),
);
const near = [0, 1, 2].map((c) => campaign.filter((s) => s[c] === ENDINGS.indexOf("NearPerson")).length);
const facts = {
  part1: { touchdownS: p1.t, driftM: Math.round(at(1, 0).x - p1.x), nearestPersonM: Math.round(p1.nearestPersonM) },
  part2: { waterPatches: Number(p2day.water_patches), stillWaterByDay: Number(p2day.still_water_by_day) },
  part3: {
    touchdownS: p3.t,
    chosenAtS: p3choice.t,
    chosenDistanceM: Math.round(dist([p3choice.x, p3choice.y], centre(p3choice.target))),
    nearestPersonM: Math.round(p3.nearestPersonM),
  },
  part4: {
    touchdownS: p4.t,
    nearestPersonM: Math.round(p4.nearestPersonM),
    heightAtCellFailureM: at(4, 85).agl,
    lookHeightM: at(4, 77).agl,
    rounds,
    lastResort: lastResort.map((r) => r.cost),
  },
  campaign: {
    scenarios: campaign.length,
    nearPerson: near,
    nearPersonPct: near.map((k) => (100 * k) / campaign.length),
    part4UpperPct: 100 * upperBound(near[2], campaign.length),
    part4NearScenario: campaign.findIndex((s) => s[2] === ENDINGS.indexOf("NearPerson")),
  },
};
check("part 1 touchdown", facts.part1.touchdownS, 91);
check("part 1 drift", facts.part1.driftM, 72);
check("part 1 nearest person", facts.part1.nearestPersonM, 71);
check("part 1 outcome", p1.outcome, "Ditched");
check("part 2 water patches", facts.part2.waterPatches, 58);
check("part 2 water by day", facts.part2.stillWaterByDay, 0);
check("part 3 chosen at", facts.part3.chosenAtS, 69);
check("part 3 chosen distance", facts.part3.chosenDistanceM, 6);
check("part 3 nearest person", facts.part3.nearestPersonM, 3);
check("part 3 outcome", p3.outcome, "AmongPeople");
check("part 4 touchdown", facts.part4.touchdownS, 90);
check("part 4 nearest person", facts.part4.nearestPersonM, 34);
check("part 4 height at the cell failure", facts.part4.heightAtCellFailureM, 13);
check("part 4 look height", facts.part4.lookHeightM, 25);
check("part 4 outcome", p4.outcome, "Safe");
check("part 4 rounds", JSON.stringify(rounds.map((r) => [r.t, r.proposals, r.permitted])), JSON.stringify([[57, 20, 0], [58, 24, 0], [64, 24, 2], [85, 72, 55]]));
check("part 4 last resort", JSON.stringify(facts.part4.lastResort), JSON.stringify([11111000, 1000, 11000]));
check("campaign near a person", JSON.stringify(near), JSON.stringify([35, 41, 1]));
check("part 4 upper bound", Math.round(facts.campaign.part4UpperPct * 100) / 100, 0.47);
check("part 4 near-person scenario", facts.campaign.part4NearScenario, 810);
inReadme("part 1 drift", "carried it 72 m down the slope");
inReadme("part 2 water", "0 of the 58 patches judged to be water");
inReadme("part 3 distance", "3 m from a member of the crew");
inReadme("part 4 touchdown", "34 m from the nearest person");
inReadme("part 4 prices", "permitted at harm cost 11111000");
inReadme("first version", "All its near-person outcomes, 3.9 %, were such falls beside the crew");
inReadme("campaign part 1", "near a person   3.5 % (at most  4.61 %)");
inReadme("campaign part 4", "near a person   0.1 % (at most  0.47 %)");
inReadme("scenario 810", "One scenario still ends near a person: 810.");
inReadme("scenario 810 cause", "The cell fails 6 s after the controller confirms the lost link, while the");
inReadme("scenario 810 reach", "lies beside ground where a person cannot be ruled out");
// The incident facts and the part 4 log lines the video quotes, as the tutorial's web pages state them.
const pages = join(root, "..", "..", "website", "web", "src", "pages", "tutorials", "dynamic-drone-failsafe");
const page = (name) => readFileSync(join(pages, name), "utf8");
const onPage = (name, label, text) => {
  if (!page(name).includes(text)) failures.push(`${label}: ${name} holds no "${text}"`);
};
onPage("index.astro", "Melbourne drones", "500 Damoda Newton V2.2 show drones launched over Victoria Harbour");
onPage("index.astro", "Melbourne wind", "Wind at more than twice their limit");
onPage("index.astro", "Melbourne harbour", "427 dropped into");
onPage("index.astro", "Orlando shift", "Setup errors had shifted the show about 18 m toward the audience");
onPage("index.astro", "Orlando boy", "A drone struck a seven-year-old boy");
onPage("effect-ethos.astro", "log round", "t=85 s: 72 proposals put to the Effect Ethos under the emergency norms: 55 permitted.");
onPage("effect-ethos.astro", "log forbidden", "t=85 s:   17 forbidden: a gust could put the drone on steep ground beside it.");
onPage("effect-ethos.astro", "log chosen", "t=85 s: Chosen: land on the patch 0 m away, at 30 m across and 430 m along; harm cost 0.");

if (failures.length > 0) {
  console.error("The video's numbers disagree with the tutorial:\n  " + failures.join("\n  "));
  process.exit(1);
}
const write = (name, value) => writeFileSync(join(out, name), JSON.stringify(value));
write("world.json", { ...worldJson, crew });
write("flights.json", flights);
write("patches.json", patchJson);
write("ethos.json", { rulings, lastResort });
write("campaign.json", campaign);
write("facts.json", facts);
console.log(`data: ${world.length} world points, ${Object.values(patchJson).flat().length} patch readings, ${rulings.length} rulings, ${campaign.length} scenarios; every number agrees with the README`);
