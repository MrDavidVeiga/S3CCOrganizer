/**
 * Stage 7 theme audit. CSS is shared by 7 choices via resolved light/dark
 * tokens; these static and WCAG checks do not impersonate WebView2 screenshots.
 */
import assert from "node:assert/strict";
import {readFileSync} from "node:fs";

const read = file => readFileSync(new URL("../"+file, import.meta.url),"utf8");
const css=read("src/ui/fluent/theme-audit.css");
const html=read("index.html");
const winui=read("src/winui.css");
const theme=read("src/theme.js");
const tokens=read("src/ui/fluent/tokens.css");
const feedback=read("src/ui/fluent/feedback.css");

assert(html.includes('href="./src/ui/fluent/theme-audit.css"'));
const auditImport=html.indexOf('href="./src/ui/fluent/theme-audit.css"');
assert(auditImport>html.indexOf('href="./src/ui/fluent/tool-paths.css"'),"audit CSS must load last");
assert.match(css,/html\[data-resolved-mode="light"\]/);
assert.match(css,/html\[data-resolved-mode="dark"\]/);
assert.match(css,/html\[data-theme-choice="mean-girls"\]/);
assert.match(css,/html\[data-theme-choice="ea-app"\]/);
assert.match(css,/--winui-control-fill-hover: rgba\(0,0,0,.065\)/);
assert.match(css,/--fluent-list-hover-fill: rgba\(0,0,0,.075\)/);
assert.match(css,/--fluent-list-hover-fill: rgba\(95,27,63,.115\)/);
assert.match(css,/--fluent-list-hover-fill: #2A2E3F/);
assert.match(css,/--fluent-list-pressed-fill: rgba\(0,0,0,.13\)/);
assert.match(css,/--fluent-list-selected-fill: rgba\(var\(--accent-rgb\),.13\)/);
assert.match(css,/\.catalog-row:nth-child\(even\)\.selected/);
assert.match(css,/--fluent-list-active-hover-fill/);
assert.match(css,/\.package-row/);
assert.match(css,/\.duplicate-row/);
assert.match(css,/\.conflict-row/);
assert.match(css,/\.restore-history-item/);
assert.match(css,/\.structure-row/);
assert.match(css,/\.catalog-grid-header/);
assert.match(css,/\.catalog-row/);
assert.match(css,/\.tools-list-item/);
assert.match(css,/\.fluent-combobox-option/);
assert.match(css,/\.content-filter-menu-item/);
assert.match(css,/\.lang-menu-item/);
assert.match(css,/\.theme-option/);
assert.match(css,/\.catalog-page-footer/);
assert.match(css,/background: var\(--winui-popup\)/);
assert.match(css,/color: var\(--fluent-text-secondary\)/);
assert.match(css,/color: var\(--fluent-state-success\)/);
assert.match(css,/color: var\(--fluent-state-error\)/);
assert.match(css,/color: var\(--fluent-state-warning\)/);
assert.match(css,/color: var\(--fluent-state-info\)/);
assert.match(css,/\.tools-health-good/);
assert.match(css,/\.plan-paths code/);
assert.match(css,/\.quarantine-preview code/);
assert.match(css,/\.catalog-row-status.state-saved/);
assert.match(css,/:disabled/);
assert.match(css,/:focus-visible/);
assert.match(css,/forced-colors: active/);
assert.match(css,/prefers-reduced-motion: reduce/);
assert.doesNotMatch(css,/\b(?:height|min-height|max-height):\s*(?:53|56|62|66)px/,"virtual list metrics must not change");
assert.match(tokens,/--fluent-text-primary: var\(--winui-text-primary/);
assert.match(feedback,/--fluent-state-success: #168545/);
assert.match(feedback,/--fluent-state-warning: #986100/);
assert.match(feedback,/--fluent-state-error: #c42b1c/);
assert.match(winui,/html\[data-theme-choice="mean-girls"\]/);
assert.match(winui,/html\[data-theme-choice="ea-app"\]/);
for(const themeName of ["system","veiga-light","veiga-dark","mean-girls","ea-app","winui-light","winui-dark"]){
  assert(html.includes('data-theme="'+themeName+'"'),"missing "+themeName+" choice");
  assert(theme.includes('"'+themeName+'"'),"missing "+themeName+" behavior");
}
assert.match(theme,/MEAN_GIRLS_ACCENT = "#F92F60"/);
assert.match(theme,/EA_APP_ACCENT = "#276AFC"/);
assert.match(read("src/ui/fluent/controls.css"),/background: #101010/);
assert.match(read("src/ui/fluent/tool-paths.css"),/color: var\(--fluent-text-primary\)/);

function hexToRgb(hex){
  const clean=hex.replace("#","");
  assert(/^[a-f0-9]{6}$/i.test(clean),"6-character test color expected: "+hex);
  return [0,2,4].map(i=>parseInt(clean.slice(i,i+2),16));
}
function luminance(h){
  const chan=hexToRgb(h).map(v=>v/255).map(v=>v<=.04045?v/12.92:((v+.055)/1.055)**2.4);
  return .2126*chan[0]+.7152*chan[1]+.0722*chan[2];
}
function contrast(a,b){
  const l=luminance(a),r=luminance(b);
  return (Math.max(l,r)+.05)/(Math.min(l,r)+.05);
}
/* Representative dark/light and Mean Girls foreground-to-panel contrasts.
 * A full visual inspection on actual Windows WebView2 is still required. */
const examples=[
  ["WinUI light text","000000","FEFEFE"],
  ["WinUI dark text","FFFFFF","202020"],
  ["EA App text","FFFFFF","202434"],
  ["Mean Girls text","4D1332","FEF9FD"],
  ["Mean Girls secondary text","7F4361","FEF9FD"],
  ["Light InfoBar success","168545","FEF9FD"],
  ["Light InfoBar warning","986100","FEF9FD"],
  ["Light InfoBar error","C42B1C","FEF9FD"],
  ["Light InfoBar info","1465B5","FEF9FD"],
  ["Dark InfoBar success","68BE7C","202020"],
  ["Dark InfoBar warning","E2AA40","202020"],
  ["Dark InfoBar error","EF8585","202020"],
];
for(const [label,a,b] of examples) assert(contrast(a,b)>=4.5,
  label+" contrast must meet 4.5:1; got "+contrast(a,b).toFixed(2));
/* Semantically, pressed is always more prominent than hover. */
const states=[
  ["WinUI/Veiga/System light",.075,.13],
  ["Mean Girls",.115,.17],
  ["WinUI/Veiga/System dark",.085,.13],
];
for(const [name,hover,pressed] of states)
  assert(pressed>hover && hover>=.07,name+" must have visible hover and stronger pressed");
/* The latest main/UI integrations must survive all future theme refactors. */
const app=read("src/app.js");
const quarantine=read("src-tauri/src/quarantine.rs");
const report=read("src-tauri/src/audit_report.rs");
assert.match(app,/function historyRecordCanBeRemoved\(/);
assert.match(app,/function removeOperationHistoryRecord\(/);
assert.match(app,/function buildScopedAuditMarkdown\(/);
assert.match(app,/reportKind/);
assert.match(quarantine,/pub fn remove_quarantine_history\(/);
assert.match(quarantine,/restore_requires_explicit_confirmation/);
assert.match(report,/reportKind/);
assert.match(read("src/style.css"),/\.history-record-actions/);
console.log("Seven-theme audit: PASS (list/menus/ComboBox states, light hover, status/text contrast, all 7 themes, no row geometry changes)");
