/* Regression smoke checks for the first WinUI foundation migration.
 * No browser mocks: runtime interactions will be covered in follow-up batches.
 */
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const read = (p) => readFileSync(new URL(`../${p}`, import.meta.url), "utf8");
const index = read("index.html");
const tokens = read("src/ui/fluent/tokens.css");
const controls = read("src/ui/fluent/controls.css");
const theme = read("src/theme.js");

const imports = [
  "src/style.css", "src/winui.css",
  "src/ui/fluent/tokens.css", "src/ui/fluent/controls.css",
];
const positions = imports.map(p=>index.indexOf(`href="./${p}"`));
assert(positions.every(p => p >= 0), "all style sheets must be imported");
assert(positions.every((p,i) => i === 0 || p > positions[i-1]), "Fluent foundation must load after existing styles");
assert.match(index, /body class="[^"]*release-v1/);
assert.match(tokens, /--fluent-radius-control: var\(--winui-control-radius, 4px\)/);
assert.match(tokens, /--fluent-radius-overlay: var\(--winui-overlay-radius, 8px\)/);
assert.match(controls, /:focus-visible/);
assert.match(controls, /input\[type="checkbox"\]/);
assert.match(controls, /:indeterminate/);
assert.match(controls, /:disabled/);
assert.match(controls, /prefers-reduced-motion/);
assert.match(controls, /forced-colors/);
assert.match(controls, /input\.text-input/);
assert.match(controls, /\.primary-btn/);
assert.match(controls, /\.secondary-btn/);
assert.match(controls, /\.danger-btn/);

for (const themeId of ["system","veiga-light","veiga-dark","mean-girls","ea-app","winui-light","winui-dark"]) {
  assert(index.includes(`data-theme="${themeId}"`), `missing ${themeId} theme choice`);
  assert(theme.includes(`"${themeId}"`), `missing ${themeId} theme logic`);
}
for (const id of ["choose-folder-btn","scan-btn","plan-btn","duplicate-details-modal","conflict-details-modal","confirm-modal","tools-readonly","tools-profile-select","duplicates-filter","conflicts-filter"]) {
  assert(index.includes(`id="${id}"`), `missing existing control ${id}`);
}
console.log("Fluent foundation static regression checks: PASS");
