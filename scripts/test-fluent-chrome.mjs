/* Combined WinUI batches 5/6 DOM interaction + static regression suite. */
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { JSDOM } from "jsdom";

const read=(path)=>readFileSync(new URL(`../${path}`,import.meta.url),"utf8");
const page=`<!doctype html><html lang="en"><body class="release-v1">
<header class="winui-titlebar">
<div class="winui-caption-buttons">
<button id="titlebar-minimize"><svg class="winui-caption-icon"></svg></button>
<button id="titlebar-maximize"><svg class="winui-caption-icon winui-maximize-glyph"></svg><svg class="winui-caption-icon winui-restore-glyph" hidden></svg></button>
<button id="titlebar-close"><svg class="winui-caption-icon"></svg></button>
</div></header>
<div class="topbar-mode-switcher" role="tablist">
<button class="mode-btn active" data-tab="organizer" aria-selected="true">Manager</button>
<button class="mode-btn" data-tab="duplicates" aria-selected="false">Duplicates</button>
<button class="mode-btn hidden" data-tab="structure" aria-selected="false">Structure</button>
<button class="mode-btn" data-tab="conflicts" aria-selected="false">Conflicts</button>
<button class="mode-btn" data-tab="tools" aria-selected="false">Tools</button>
<button class="mode-btn" data-tab="restore" aria-selected="false">Restore</button>
</div>
${["organizer","duplicates","structure","conflicts","tools","restore"].map(id=>
  `<section class="tool-page" id="page-${id}"></section>`).join("")}
<div class="results-query-controls">
<input id="search-input" type="search" class="text-input" value="initial" />
</div><input id="duplicates-search" type="search" class="text-input">
<input id="conflicts-search" type="search" class="text-input">
<input id="catalog-search" type="search" class="text-input">
<details id="diagnostics-panel" class="sidebar-diagnostics"><summary>Performance</summary><p>Content</p></details>
<button id="tooltip-target" title="A descriptive WinUI tooltip">ToolTip</button>
</body></html>`;
const dom=new JSDOM(page,{url:"https://localhost/",pretendToBeVisual:true,runScripts:"outside-only"});
const {window}=dom,{document}=window;
const evaluate=path=>{
 const source=read(path).replace(/^export\s+\{[^}]*\};?\s*$/gm,"");
 window.eval(source);
};
const wait=()=>new Promise(resolve=>setTimeout(resolve,0));

let inputCount=0,changeCount=0;
const search=document.getElementById("search-input");
search.addEventListener("input",()=>inputCount++);
search.addEventListener("change",()=>changeCount++);
evaluate("src/ui/fluent/fields.js");
assert.equal(document.querySelectorAll(".fluent-search-box").length,4);
assert.equal(search.id,"search-input","existing application ID must be untouched");
const wrap=search.closest(".fluent-search-box"),clear=wrap.querySelector(".fluent-search-clear");
assert(clear);
assert.equal(clear.hidden,false);
clear.click();
assert.equal(search.value,"");
assert.equal(inputCount,1,"clear must fire native input");
assert.equal(changeCount,1,"clear must fire native change");
assert.equal(clear.hidden,true);
assert.equal(document.activeElement,search,"clear returns focus to search");
search.value="another";
search.dispatchEvent(new window.Event("input",{bubbles:true}));
assert.equal(clear.hidden,false);
document.documentElement.lang="pt";
window.dispatchEvent(new window.CustomEvent("s3cc-language-changed"));
assert.equal(clear.getAttribute("aria-label"),"Limpar pesquisa");
search.disabled=true;
await wait();
assert.equal(clear.hidden,true);
search.disabled=false;
await wait();

const tabs=[...document.querySelectorAll(".mode-btn[data-tab]")];
let selected="organizer",clicks=0;
for(const tab of tabs)tab.addEventListener("click",()=>{
  clicks++;
  selected=tab.dataset.tab;
  tabs.forEach(b=>{
    const current=b===tab;
    b.classList.toggle("active",current);
    b.setAttribute("aria-selected",String(current));
  });
});
evaluate("src/ui/fluent/chrome.js");
assert.equal(tabs[0].getAttribute("role"),"tab");
assert.equal(tabs[0].tabIndex,0);
assert.equal(tabs[1].tabIndex,-1);
assert.equal(document.getElementById("page-organizer").getAttribute("role"),"tabpanel");
assert.equal(document.getElementById("page-organizer").getAttribute("aria-labelledby"),tabs[0].id);
const key=(element,value)=>{
  const e=new window.KeyboardEvent("keydown",{key:value,cancelable:true,bubbles:true});
  element.dispatchEvent(e);return e;
};
tabs[0].focus();
let keypress=key(tabs[0],"ArrowRight");
assert.equal(keypress.defaultPrevented,true);
assert.equal(selected,"duplicates");
assert.equal(clicks,1);
assert.equal(document.activeElement,tabs[1]);
await wait();
assert.equal(tabs[1].tabIndex,0);
keypress=key(tabs[1],"ArrowRight");
assert.equal(selected,"conflicts","hidden Structure tab must be skipped");
keypress=key(tabs[3],"End");
assert.equal(selected,"restore");
keypress=key(tabs[5],"Home");
assert.equal(selected,"organizer");
keypress=key(tabs[0],"ArrowLeft");
assert.equal(selected,"restore","left Arrow wraps around");
assert.equal(tabs[5].getAttribute("aria-selected"),"true");

const target=document.getElementById("tooltip-target");
evaluate("src/ui/fluent/tooltips.js");
target.dispatchEvent(new window.MouseEvent("pointerover",{bubbles:true}));
const tooltip=document.getElementById("fluent-shared-tooltip");
assert.equal(tooltip.hidden,false);
assert.equal(tooltip.textContent,"A descriptive WinUI tooltip");
assert.equal(target.getAttribute("title"),null,"avoid native overlapping tooltip");
assert.equal(target.getAttribute("aria-describedby"),tooltip.id);
target.dispatchEvent(new window.MouseEvent("pointerout",{bubbles:true}));
assert.equal(tooltip.hidden,true);
assert.equal(target.title,"A descriptive WinUI tooltip");
target.focus();
assert.equal(tooltip.hidden,false,"keyboard focus opens tooltip");
key(target,"Escape");
assert.equal(tooltip.hidden,true);

const details=document.getElementById("diagnostics-panel");
details.open=true;
assert.equal(details.open,true,"Expander must retain native open state");
details.open=false;
assert.equal(details.open,false);

const original=read("index.html");
for(const id of ["minimize","maximize","close"]) {
 const marker=new RegExp(`id="titlebar-${id}"[^]*?<svg class="winui-caption-icon`);
 assert.match(original,marker,"caption button uses vector icon");
}
assert.doesNotMatch(original,/titlebar-maximize"[^\n]*>□</);
assert.match(original,/src\/ui\/fluent\/expanders\.css/);
assert.match(original,/src\/ui\/fluent\/fields\.css/);
assert.match(original,/src\/ui\/fluent\/chrome\.css/);
assert.match(original,/src\/ui\/fluent\/tooltips\.css/);
assert.match(original,/src\/ui\/fluent\/chrome\.js/);
assert.match(original,/src\/ui\/fluent\/tooltips\.js/);
// Mean Girls campaign's Get Kit CTA: black-on-white? No: BLACK button,
 // WHITE foreground. Preserve EA App standard blue/white independently.
const buttonStyles=read("src/ui/fluent/controls.css");
const meanGirlsPrimary=buttonStyles.match(/html\[data-theme-choice="mean-girls"\] body\.release-v1 \.primary-btn \{([^}]*)\}/);
assert(meanGirlsPrimary,"Mean Girls should have its own primary CTA declaration");
assert.match(meanGirlsPrimary[1],/background: #101010/);
assert.match(meanGirlsPrimary[1],/color: #ffffff/);
assert.match(buttonStyles,/html\[data-theme-choice="mean-girls"\] body\.release-v1 \.primary-btn:hover:not\(:disabled\) \{[^}]*background: #292929/);
assert.match(buttonStyles,/html\[data-theme-choice="mean-girls"\] body\.release-v1 \.primary-btn:active:not\(:disabled\) \{[^}]*background: #050505/);
const brandTheme=read("src/theme.js");
const brandWinui=read("src/winui.css");
assert.match(brandTheme,/EA_APP_ACCENT = "#276AFC"/);
assert.match(brandWinui,/html\[data-theme-choice="ea-app"\] \.release-v1 \.primary-btn \{[^}]*background: var\(--accent\)/);
assert.match(brandWinui,/html\[data-theme-choice="mean-girls"\] \.release-v1 \.mode-btn\.active::after \{[^}]*#F92F60/);
const theme=read("src/theme.js");
assert.match(theme,/updateCaptionMaximize/);
assert.match(theme,/onResized/);
const css=[read("src/ui/fluent/chrome.css"),read("src/ui/fluent/expanders.css"),read("src/ui/fluent/fields.css"),read("src/ui/fluent/tooltips.css")].join("\n");
assert.match(css,/prefers-reduced-motion/);
assert.match(css,/forced-colors/);
console.log("Fluent combined Stage 5+6 tests: PASS (SearchBox clear/input, native Expanders, tab keyboard navigation, SVG captions, tooltip focus/pointer, accessible states)");
dom.window.close();
