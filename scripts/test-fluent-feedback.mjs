/* Browser-like semantic feedback tests using JSDOM (WebView visual QA remains manual). */
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { JSDOM } from "jsdom";

const read=p=>readFileSync(new URL(`../${p}`,import.meta.url),"utf8");
const names=["scan","duplicates","conflicts"];
const html=`<!doctype html><html lang="en"><body class="release-v1">
${names.map(n=>`<div id="${n}-state" class="scan-state"></div>
<div id="${n}-progress" class="operation-progress hidden">
  <div class="operation-progress-topline">
    <span id="${n}-progress-text"></span><strong id="${n}-progress-percent">0%</strong>
  </div>
  <div class="operation-progress-track" role="progressbar" aria-valuemin="0" aria-valuemax="100">
    <span id="${n}-progress-bar"></span>
  </div>
  <div id="${n}-progress-count">0 / 0</div><button id="${n}-cancel-btn">Cancel</button>
</div>`).join("")}
<div id="tools-state" class="scan-state"></div>
<div id="restore-state" class="scan-state"></div>
<div id="sims3pack-converter-state" class="scan-state"></div>
</body></html>`;
const dom=new JSDOM(html,{url:"https://localhost/",runScripts:"outside-only",pretendToBeVisual:true});
const {window}=dom,{document}=window;
const flush=()=>new Promise(done=>setTimeout(done,0));
const feedbackSource=read("src/ui/fluent/feedback.js").replace(/^export\s+\{[^}]*\};?\s*$/gm,"");
window.eval(feedbackSource);

const scan=document.getElementById("scan-state");
assert(scan.classList.contains("fluent-infobar"));
assert.equal(scan.getAttribute("role"),null,"empty status should not be announced");
scan.textContent="Analyzing packages";
scan.className="scan-state busy";
await flush();
assert.equal(scan.dataset.fluentTone,"busy");
assert.equal(scan.getAttribute("role"),"status");
assert.equal(scan.getAttribute("aria-busy"),"true");
scan.className="scan-state error";
scan.textContent="Operation failed";
await flush();
assert.equal(scan.dataset.fluentTone,"error");
assert.equal(scan.getAttribute("role"),"alert");
assert.equal(scan.getAttribute("aria-live"),"assertive");
scan.className="scan-state success";
scan.textContent="All done";
await flush();
assert.equal(scan.dataset.fluentTone,"success");
scan.dataset.fluentSeverity="warning";
await flush();
assert.equal(scan.dataset.fluentTone,"warning","structured warnings override success");
scan.dataset.fluentSeverity="";
await flush();
assert.equal(scan.dataset.fluentTone,"success");
scan.textContent="";
await flush();
assert.equal(scan.hasAttribute("role"),false);
assert.equal(scan.hasAttribute("data-fluent-tone"),false);

const appSource=read("src/app.js");
const start=appSource.indexOf("function renderOperationProgress(kind) {");
const end=appSource.indexOf("function renderAllOperationProgress() {",start);
assert(start>=0&&end>start,"must find real application progress renderer");
const operationRenderer=appSource.slice(start,end);
const opStates={
  scan:{running:true,total:100,processed:25,phase:"scanning",cancelRequested:false,current:"PRIVATE_FILE.package"},
  duplicates:{running:true,total:0,processed:0,phase:"starting",cancelRequested:false},
  conflicts:{running:true,total:9,processed:9,phase:"comparing",cancelRequested:false},
};
window.testOperations=opStates;
window.eval(`(()=>{
const state={operations:window.testOperations,scanning:true,duplicatesBusy:true,conflictsBusy:true};
const el={};
for(const n of ["scan","duplicates","conflicts"]){
  const prefix=n.replace(/^./,letter=>letter.toUpperCase());
  const label=n==="scan"?"scan":"";
  el[n==="scan"?"scanProgress":n+"Progress"]=document.getElementById(n+"-progress");
  el[n==="scan"?"scanProgressBar":n+"ProgressBar"]=document.getElementById(n+"-progress-bar");
  el[n==="scan"?"scanProgressText":n+"ProgressText"]=document.getElementById(n+"-progress-text");
  el[n==="scan"?"scanProgressPercent":n+"ProgressPercent"]=document.getElementById(n+"-progress-percent");
  el[n==="scan"?"scanProgressCount":n+"ProgressCount"]=document.getElementById(n+"-progress-count");
  el[n==="scan"?"scanCancelBtn":n+"CancelBtn"]=document.getElementById(n+"-cancel-btn");
}
const operationUi=kind=>({
 scan:[el.scanProgress,el.scanProgressBar,el.scanProgressText,el.scanProgressPercent,el.scanProgressCount,el.scanCancelBtn],
 duplicates:[el.duplicatesProgress,el.duplicatesProgressBar,el.duplicatesProgressText,el.duplicatesProgressPercent,el.duplicatesProgressCount,el.duplicatesCancelBtn],
 conflicts:[el.conflictsProgress,el.conflictsProgressBar,el.conflictsProgressText,el.conflictsProgressPercent,el.conflictsProgressCount,el.conflictsCancelBtn]
})[kind];
const operationBusy=kind=>({scan:state.scanning,duplicates:state.duplicatesBusy,conflicts:state.conflictsBusy})[kind];
const operationPhaseLabel=phase=>({scanning:"Scanning packages",starting:"Preparing",comparing:"Comparing"})[phase]||"Preparing";
const integerLabel=value=>Number(value||0).toLocaleString("en-US");
const t=key=>({cancelling:"Cancelling",cancelAnalysis:"Cancel"})[key]||key;
${operationRenderer}
window.testRenderOperationProgress=renderOperationProgress;
window.testSetBusy=(kind,value)=>{if(kind==="scan")state.scanning=value;else if(kind==="duplicates")state.duplicatesBusy=value;else state.conflictsBusy=value};
})();`);
const r=window.testRenderOperationProgress;
r("scan");
const s=document.getElementById("scan-progress");
const track=s.querySelector('[role="progressbar"]');
assert.equal(s.classList.contains("hidden"),false);
assert.equal(s.dataset.fluentProgress,"determinate");
assert.equal(track.getAttribute("aria-valuenow"),"25");
assert.equal(track.getAttribute("aria-valuetext"),"25% — 25 / 100");
assert.equal(document.getElementById("scan-progress-bar").style.width,"25%");
assert.equal(document.getElementById("scan-progress-percent").textContent,"25%");
assert.equal(s.textContent.includes("PRIVATE_FILE.package"),false,"progress must not expose individual filenames");

r("duplicates");
const unknown=document.getElementById("duplicates-progress");
const unknownTrack=unknown.querySelector('[role="progressbar"]');
assert.equal(unknown.dataset.fluentProgress,"indeterminate");
assert.equal(unknownTrack.hasAttribute("aria-valuenow"),false);
assert.equal(unknownTrack.getAttribute("aria-valuetext"),"Preparing");
assert.equal(document.getElementById("duplicates-progress-percent").textContent,"—");
assert.equal(document.getElementById("duplicates-progress-bar").style.width,"");

opStates.conflicts.cancelRequested=true;
r("conflicts");
assert.equal(document.getElementById("conflicts-progress-cancel-btn"),null); // native button uses conflicts-cancel-btn
assert.equal(document.getElementById("conflicts-cancel-btn").disabled,true);
assert.equal(document.getElementById("conflicts-cancel-btn").textContent,"Cancelling");
assert.equal(document.getElementById("conflicts-progress").querySelector('[role="progressbar"]').getAttribute("aria-valuenow"),"100");

window.testSetBusy("scan",false);
opStates.scan.running=false;
r("scan");
assert.equal(s.classList.contains("hidden"),true);
const css=read("src/ui/fluent/feedback.css");
assert.match(css,/fluent-progress-travel/);
assert.match(css,/fluent-ring-spin/);
assert.match(css,/data-fluent-tone="warning"/);
assert.match(css,/prefers-reduced-motion/);
assert.match(css,/forced-colors/);
const htmlSource=read("index.html");
assert.equal((htmlSource.match(/role="progressbar"/g)||[]).length,4);
assert.match(htmlSource,/src\/ui\/fluent\/feedback\.css/);
assert.match(htmlSource,/src\/ui\/fluent\/feedback\.js/);
assert.match(appSource,/stats\.analysisTruncated/);
assert.match(appSource,/stats\.variantAnalysisTruncated/);
console.log("Fluent feedback tests: PASS (determinate/indeterminate progress, accessibility, busy ring, warning/error InfoBars, privacy, cancellation)");
dom.window.close();
