//
// SPDX-License-Identifier: MIT
// Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
//
// Usage: node scripts/sync_pnpm_overrides.mjs <site directory>
//
// Moves each override in the site's pnpm-workspace.yaml to the exact version a direct dependency
// pins for that package: the site's own package.json, or the installed package.json of a direct
// dependency. `pnpm update --latest` moves those pins but never the override, which then either
// contradicts the site's own pin (the lockfile fails a frozen install) or forces a version the
// dependency does not declare.
//
// An override nothing pins exactly is left alone. Two different exact pins for one package exit
// with status 1: no single override satisfies both.

import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const site = process.argv[2];
const manifest = JSON.parse(readFileSync(join(site, 'package.json'), 'utf8'));
const direct = { ...manifest.dependencies, ...manifest.devDependencies };
const exact = /^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/;

// Every exact pin of `name`, mapped to the packages that pin it.
function exactPins(name) {
    const pins = new Map();
    const add = (version, by) => pins.set(version, [...(pins.get(version) ?? []), by]);
    if (exact.test(direct[name] ?? '')) add(direct[name], 'package.json');
    for (const dep of Object.keys(direct)) {
        const installed = JSON.parse(readFileSync(join(site, 'node_modules', dep, 'package.json'), 'utf8'));
        const spec = installed.dependencies?.[name];
        if (exact.test(spec ?? '')) add(spec, `${dep}@${installed.version}`);
    }
    return pins;
}

const workspacePath = join(site, 'pnpm-workspace.yaml');
const lines = readFileSync(workspacePath, 'utf8').split('\n');
const entry = /^  (?:'([^']+)'|"([^"]+)"|([^\s:'"]+)): (\S+)\s*$/;
let inOverrides = false;
let changed = false;

for (const [i, line] of lines.entries()) {
    if (/^overrides:\s*$/.test(line)) {
        inOverrides = true;
        continue;
    }
    if (/^\S/.test(line)) inOverrides = false;
    const match = inOverrides && line.match(entry);
    if (!match) continue;

    const name = match[1] ?? match[2] ?? match[3];
    const current = match[4];
    const pins = exactPins(name);

    if (pins.size > 1) {
        const detail = [...pins].map(([version, by]) => `  ${version} by ${by.join(', ')}`).join('\n');
        console.error(`Conflicting exact pins for override '${name}':\n${detail}`);
        process.exit(1);
    }
    const [version] = pins.keys();
    if (version !== undefined && version !== current) {
        lines[i] = line.replace(/: \S+\s*$/, `: ${version}`);
        console.log(`override ${name}: ${current} -> ${version}`);
        changed = true;
    }
}

if (changed) writeFileSync(workspacePath, lines.join('\n'));
