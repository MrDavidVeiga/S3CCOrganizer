/* Regression: Primary, Secondary and Danger differ by style, NOT geometry.
 * DOM CSS tests are intentionally small; the Windows WebView2 visual pass
 * remains necessary to confirm physical sizing and icon alignment. */
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { JSDOM } from "jsdom";

const read = path => readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const markup = `<!doctype html><html data-theme-choice="winui-dark">
<head></head><body class="release-v1">
  <aside class="sidebar"><button id="choose-folder-btn" class="primary-btn">Choose Mods Folder</button></aside>
  <div class="sims3pack-converter-actions">
    <button id="choose-files" class="secondary-btn"><i aria-hidden="true"></i><span>Choose Sims3Pack(s)</span></button>
    <button id="choose-destination" class="secondary-btn"><i aria-hidden="true"></i><span>Choose Destination</span></button>
    <button id="convert" class="primary-btn"><i aria-hidden="true"></i><span>Convert to .package</span></button>
  </div>
  <div class="restore-history-actions">
    <button id="remove" class="danger-btn">Remove Record</button>
    <button id="refresh" class="secondary-btn">Refresh</button>
  </div>
  <div class="confirm-actions">
    <button class="secondary-btn">Cancel</button>
    <button class="primary-btn">Confirm</button>
  </div>
  <div id="page-organizer">
    <div class="results-actions">
      <div class="results-query-controls"><input id="search-input"></div>
      <div class="results-selection-controls">
        <button id="select-all-btn" class="secondary-btn">Selecionar Tudo</button>
        <button id="select-none-btn" class="secondary-btn">Selecionar Nenhum</button>
      </div>
    </div>
  </div>
  <div class="sidebar-utility-actions">
    <button id="compact" class="compact-btn">Utility</button>
  </div>
</body></html>`;
const dom = new JSDOM(markup,{url:"https://localhost/",pretendToBeVisual:true});
const {window}=dom,{document}=window;
const geometry = read("src/ui/fluent/button-geometry.css");
const legacy = `
.primary-btn {padding:12px 14px;font-weight:700}
.secondary-btn {padding:9px 11px;font-weight:700}
.danger-btn {padding:10px 13px;font-weight:700}
`;
const style=document.createElement("style");
style.textContent=legacy+"\n"+geometry;
document.head.appendChild(style);
const ids=["choose-files","choose-destination","convert","remove","refresh"];
const get = id => window.getComputedStyle(document.getElementById(id));
const metrics = id => ({
  paddingTop:get(id).paddingTop,
  paddingBottom:get(id).paddingBottom,
  paddingLeft:get(id).paddingLeft,
  paddingRight:get(id).paddingRight,
  borderTop:get(id).borderTopWidth,
  borderBottom:get(id).borderBottomWidth,
  weight:get(id).fontWeight,
  lineHeight:get(id).lineHeight,
  display:get(id).display,
  minHeight:get(id).minHeight,
});
const first = metrics(ids[0]);
for (const id of ids.slice(1)) {
  assert.deepEqual(metrics(id),first,`geometry must match for ${id}`);
}
assert.equal(first.paddingTop,"5px");
assert.equal(first.paddingBottom,"5px");
assert.equal(first.paddingLeft,"12px");
assert.equal(first.paddingRight,"12px");
assert.equal(first.borderTop,"1px");
assert.equal(first.borderBottom,"1px");
assert.equal(first.weight,"600");
assert.equal(first.lineHeight,"20px");
assert.equal(first.display,"inline-flex");
// Language expansion must never make the right-toolbar pair two lines.
for(const id of ["select-all-btn","select-none-btn"]){
  const selected=get(id);
  assert.equal(selected.whiteSpace,"nowrap",id+" stays on one line in PT");
  assert.equal(selected.paddingLeft,"7px");
  assert.equal(selected.paddingRight,"7px");
  assert.equal(selected.paddingTop,"5px");
  assert(["14px","var(--fluent-control-font-size)"].includes(selected.fontSize), "font remains the Fluent 14px token");
}
assert.match(geometry, /#page-organizer \.results-selection-controls \{/);
assert.match(geometry, /grid-column: 1 \/ -1/);
assert.match(geometry, /grid-template-columns: repeat\(2, max-content\)/);
assert.equal(get("choose-folder-btn").paddingTop,"12px","sidebar large button stays large");
assert.equal(get("choose-folder-btn").paddingLeft,"14px");
assert.equal(get("compact").fontSize,"12px","compact sidebar actions are intentional");
for (const className of ["primary-btn","secondary-btn","danger-btn","compact-btn"]){
  assert.match(geometry,new RegExp("\\."+className+"(?:,|\\s)"));
}
assert.match(geometry,/min-height: 52px/);
assert.match(geometry,/font-size: var\(--fluent-control-font-size\)/);
assert.match(geometry,/font-family: var\(--fluent-body-font\)/);
assert.match(geometry,/border-radius: var\(--fluent-radius-control\)/);
assert.match(geometry,/prefers-reduced-motion/);
assert.match(geometry,/forced-colors/);
const token = read("src/ui/fluent/tokens.css");
assert.match(token,/--fluent-control-height: 32px/);
assert.match(token,/--fluent-control-font-size: 14px/);
const html = read("index.html");
const importPos=html.indexOf('href="./src/ui/fluent/button-geometry.css"');
assert(importPos>html.indexOf('href="./src/ui/fluent/controls.css"'),"geometry must load after original control rules");
assert(importPos>html.indexOf('href="./src/ui/fluent/tooltips.css"'),"geometry must be last CSS import");
const theme = read("src/theme.js");
assert.match(theme,/EA_APP_ACCENT = "#276AFC"/);
assert.match(theme,/MEAN_GIRLS_ACCENT/);
const controls = read("src/ui/fluent/controls.css");
assert.match(controls,/html\[data-theme-choice="mean-girls"\] body\.release-v1 \.primary-btn \{[^}]*background: #101010/);
assert.match(controls,/html\[data-theme-choice="mean-girls"\] body\.release-v1 \.primary-btn \{[^}]*color: #ffffff/);
const winui = read("src/winui.css");
assert.match(winui,/html\[data-theme-choice="ea-app"\] \.release-v1 \.primary-btn \{[^}]*background: var\(--accent\)/);
console.log("Fluent Button geometry: PASS (Converter, Restore, Confirm, large sidebar, compact exception, all themes)");
dom.window.close();
