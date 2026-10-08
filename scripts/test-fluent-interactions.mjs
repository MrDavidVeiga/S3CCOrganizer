/* Browser-like tests for ComboBox/menu/dialog behaviour.
 * Uses JSDOM as a DOM simulator. Runtime WebView visual tests remain mandatory.
 */
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { JSDOM } from "jsdom";

const read=p=>readFileSync(new URL(`../${p}`,import.meta.url),"utf8");
const options=Array.from({length:1200},(_,i)=>`<option value="v${i}">Package ${i}</option>`).join("");
const html=`<!doctype html><html lang="en"><body class="release-v1">
  <div class="app-shell">
    <button id="launcher">Open dialog</button>
    <label for="test-select">Category</label>
    <select id="test-select" class="text-input">
      <option value="a">Alpha</option><option value="b">Beta</option><option value="c">Gamma</option>
    </select>
    <select id="big-select" class="text-input">${options}</select>
    <button id="language-button" aria-haspopup="listbox" aria-expanded="false">Language</button>
    <div id="language-menu" class="hidden" role="listbox"><button data-lang="en">EN</button><button data-lang="es">ES</button></div>
    <button id="status-filter-toggle-btn">Filter</button><div id="status-filter-menu" class="hidden" role="listbox"><button class="active">All</button><button>Valid</button></div>
    <button id="theme-button">Theme</button><div id="theme-flyout" class="hidden" role="menu"><button role="menuitemradio" aria-checked="true">System</button><button role="menuitemradio" aria-checked="false">Dark</button></div>
    <div class="tabs"><button class="mode-btn active">Manager</button></div>
  </div>
  <div class="modal-backdrop hidden" id="plan-modal" aria-hidden="true">
    <section role="dialog" aria-labelledby="plan-title"><h2 id="plan-title">Plan</h2><input id="plan-field"/><button id="plan-close-btn">Close Plan</button><button id="open-nested">Confirm</button></section>
  </div>
  <div class="modal-backdrop hidden" id="confirm-modal" aria-hidden="true">
    <section role="dialog" aria-labelledby="confirm-title"><h2 id="confirm-title">Confirm</h2><button id="confirm-cancel-btn">Cancel</button><button id="confirm-action-btn">Act</button></section>
  </div>
</body></html>`;

const dom=new JSDOM(html,{url:"https://localhost/",runScripts:"outside-only",pretendToBeVisual:true});
const {window}=dom;const {document}=window;
window.HTMLElement.prototype.getClientRects=function(){
  return this.closest(".hidden,[hidden]")?[]:[{top:0,bottom:30,left:0,right:200,width:200,height:30}];
};
function evaluate(path){
  const source=read(path)
    .replace(/^import .*?;\s*$/gm,"")
    .replace(/^export function (\w+)\(/gm,"window.$1 = function $1(");
  window.eval(`(()=>{\n${source}\n})();`);
}
const flush=()=>new Promise(resolve=>setTimeout(resolve,0));
const send=(element,key,shiftKey=false)=>element.dispatchEvent(new window.KeyboardEvent("keydown",{key,shiftKey,bubbles:true,cancelable:true}));

evaluate("src/ui/fluent/combobox.js");
const select=document.getElementById("test-select");
const trigger=document.getElementById("fluent-combobox-test-select-trigger");
const popup=document.getElementById("fluent-combobox-flyout");
assert.equal(trigger.getAttribute("role"),"combobox");
assert.equal(select.options.length,3);
assert.equal(trigger.textContent.includes("Alpha"),true);
assert.equal(select.getAttribute("aria-hidden"),"true");
let changes=0;select.addEventListener("change",()=>changes++);
trigger.click();
assert.equal(trigger.getAttribute("aria-expanded"),"true");
assert.equal(popup.hidden,false);
send(trigger,"ArrowDown");
send(trigger,"Enter");
assert.equal(select.value,"b","keyboard must update original select value");
assert.equal(changes,1,"native change event must still fire");
assert.equal(trigger.getAttribute("aria-expanded"),"false");
assert(trigger.textContent.includes("Beta"));
trigger.click();
const gamma=[...popup.querySelectorAll('[role="option"]')].find(e=>e.textContent.includes("Gamma"));
assert(gamma,"visible option must be rendered");
gamma.click();
assert.equal(select.value,"c","pointer option selection must work");
assert.equal(changes,2);

// A CLOSED WinUI ComboBox changes selection immediately with arrow keys.
// It must not expand the popup or bypass the original native change event.
send(trigger,"ArrowUp");
assert.equal(select.value,"b","closed ArrowUp selects the previous entry");
assert.equal(popup.hidden,true,"closed arrows do not expand ComboBox");
assert.equal(changes,3);
send(trigger,"ArrowDown");
assert.equal(select.value,"c","closed ArrowDown selects the next entry");
assert.equal(popup.hidden,true);
assert.equal(changes,4);

// Pointer hover updates visible highlight without replacing the hovered node.
trigger.click();
const betaOption=[...popup.querySelectorAll('[role="option"]')].find(e=>e.textContent.includes("Beta"));
assert(betaOption);
betaOption.dispatchEvent(new window.Event("pointerenter"));
assert.equal(betaOption.classList.contains("active"),true,"pointer hover must visibly highlight option");
assert.equal(trigger.getAttribute("aria-activedescendant"),betaOption.id);
assert.equal(popup.querySelectorAll('.fluent-combobox-option.active').length,1);
send(trigger,"Escape");
assert.equal(popup.hidden,true);
const delta=document.createElement("option");delta.value="d";delta.textContent="Delta";select.append(delta);
await flush();
trigger.click();
send(trigger,"d");
send(trigger,"Enter");
assert.equal(select.value,"d","dynamic option changes must be available");
assert.equal(changes,5);

const big=document.getElementById("big-select");
big.value="v900";
const bigTrigger=document.getElementById("fluent-combobox-big-select-trigger");
bigTrigger.click();
assert(popup.querySelectorAll('[role="option"]').length<80,"long option lists must be virtualized");
send(bigTrigger,"End");
send(bigTrigger,"Enter");
assert.equal(big.value,"v1199");
assert.equal(popup.hidden,true);
send(bigTrigger,"ArrowUp");
assert.equal(big.value,"v1198","virtual list closed arrows select immediately");
assert.equal(popup.hidden,true);
send(bigTrigger,"ArrowDown");
assert.equal(big.value,"v1199");
assert.equal(popup.hidden,true);

const language=document.getElementById("language-menu");
const languageTrigger=document.getElementById("language-button");
languageTrigger.addEventListener("click",()=>language.classList.toggle("hidden"));
const theme=document.getElementById("theme-flyout");
document.getElementById("theme-button").addEventListener("click",()=>theme.classList.toggle("hidden"));
const status=document.getElementById("status-filter-menu");
document.getElementById("status-filter-toggle-btn").addEventListener("click",()=>status.classList.toggle("hidden"));
evaluate("src/ui/fluent/menus.js");
assert.equal(language.querySelector("button").getAttribute("role"),"option");
send(languageTrigger,"ArrowDown");
await flush();
assert.equal(language.classList.contains("hidden"),false);
assert.equal(document.activeElement,language.querySelector('[data-lang="en"]'));
send(document.activeElement,"ArrowDown");
assert.equal(document.activeElement,language.querySelector('[data-lang="es"]'));
send(document.activeElement,"Escape");
assert.equal(language.classList.contains("hidden"),true);
assert.equal(document.activeElement,languageTrigger);

const plan=document.getElementById("plan-modal");
const confirm=document.getElementById("confirm-modal");
document.getElementById("plan-close-btn").addEventListener("click",()=>plan.classList.add("hidden"));
document.getElementById("confirm-cancel-btn").addEventListener("click",()=>confirm.classList.add("hidden"));
evaluate("src/ui/fluent/overlays.js");
const launcher=document.getElementById("launcher");
launcher.focus();plan.classList.remove("hidden");await flush();
assert.equal(window.activeFluentDialogId(),"plan-modal");
assert.equal(document.activeElement.id,"plan-field","dialog focuses initial field");
assert.equal(document.querySelector(".app-shell").inert,true);
confirm.classList.remove("hidden");await flush();
assert.equal(window.activeFluentDialogId(),"confirm-modal");
assert.equal(document.activeElement.id,"confirm-cancel-btn");
send(document.activeElement,"Escape");await flush();
assert.equal(confirm.classList.contains("hidden"),true);
assert.equal(window.activeFluentDialogId(),"plan-modal");
assert.equal(document.activeElement.id,"plan-field","nested dialog restores prior focus");
send(document.activeElement,"Escape");await flush();
assert.equal(plan.classList.contains("hidden"),true);
assert.equal(window.activeFluentDialogId(),null);
assert.equal(document.querySelector(".app-shell").inert,false);
assert.equal(document.activeElement,launcher,"closing parent dialog restores launcher focus");

// Guard against regressing EA App to coral; also ensure list row sizes do
// not drift away from the virtualization stride in app.js.
const themeSource=read("src/theme.js");
const tokensSource=read("src/ui/fluent/tokens.css");
const swatchSource=read("src/winui.css");
const collectionStyles=read("src/ui/fluent/collections.css");
const comboStyles=read("src/ui/fluent/combobox.css");
const htmlSource=read("index.html");
assert.match(themeSource, /EA_APP_ACCENT = "#276AFC"/);
assert.match(themeSource, /light1: "#3978FC"/);
assert.match(themeSource, /dark1: "#215BD8"/);
assert.match(swatchSource, /\.ea-app-swatch[^]*?#276AFC/);
assert.match(tokensSource, /html\[data-theme-choice="ea-app"\][^]*?--fluent-accent-foreground: #ffffff/);
assert.match(comboStyles, /\.fluent-combobox-option:hover:not\(\.disabled\)/);
assert.match(collectionStyles, /height: 53px/);
assert.match(collectionStyles, /height: 56px/);
assert.match(htmlSource, /src\/ui\/fluent\/collections\.css/);
console.log("Fluent interaction tests: PASS (closed-arrow selection, pointer hover, EA App blue, aligned rows, virtualization, menus, nested dialogs, focus)");
dom.window.close();
