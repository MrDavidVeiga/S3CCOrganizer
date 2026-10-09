// Regression: organization must not re-read the full DBPF collection on
// success; reconcile confirmed source/destination moves instead.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import vm from "node:vm";
const read = p => readFileSync(new URL("../" + p, import.meta.url), "utf8");
const js = read("src/app.js");
const html = read("index.html");
const executor = read("src-tauri/src/executor.rs");
const scanner = read("src-tauri/src/scanner.rs");
const begin = js.indexOf("async function executeOrganization() {");
const end = js.indexOf("async function chooseManifest() {", begin);
assert(begin !== -1 && end > begin);
const execute = js.slice(begin, end);
assert(!/await\s+scanFolder\s*\(/.test(execute), "Organization must not force a complete rescan");
assert(/reconcileSuccessfulOrganization\(result\)/.test(execute));
assert(/await loadRestoreHistory\(\)/.test(execute));
assert(html.includes('id="open-organized-folder-btn"'));
assert(html.includes('id="open-organized-result-btn"'));
assert(html.includes('class="conflicts-extra-actions"'));
assert(html.includes('id="conflicts-preview-quarantine-btn"'));
assert(/invalidate_latest_scan_for\(&root\)/.test(executor));
assert(/pub fn invalidate_latest_scan_for/.test(scanner));
assert(/pub moved_files: Vec<OrganizationMove>/.test(executor));

const a = js.indexOf("function reconcileSuccessfulOrganization(result) {");
const b = js.indexOf("async function executeOrganization() {", a);
assert(a > 0 && b > a);
const state = {
  items: [
    {id:"C:\\Mods\\Packages\\Old\\one.package",path:"C:\\Mods\\Packages\\Old\\one.package",relativePath:"Old\\one.package",status:"classified"},
    {id:"C:\\Mods\\Packages\\Old\\two.package",path:"C:\\Mods\\Packages\\Old\\two.package",relativePath:"Old\\two.package",status:"unknown"},
  ],
  selectedId:"C:\\Mods\\Packages\\Old\\one.package",
  selectedForPlan:new Set(["C:\\Mods\\Packages\\Old\\one.package"]),
  plan:{items:[1]},planError:"",packagePreviews:{"old":"thumb"},
  packagePreviewLoading:{},packagePreviewErrors:{},
  technicalDetails:{"old":"dbpf"},technicalDetailsLoading:"",technicalDetailsErrors:{},
  technicalDetailsOpen:new Set(["old"]),
  analysisRunId:0,analysisStatus:{manager:"completed",duplicates:"completed",conflicts:"completed"},
  duplicatesAnalysis:{groups:[]},conflictsAnalysis:{findings:[]},
  duplicatesError:"",conflictsError:"",duplicatesNotice:"",conflictsNotice:"",
  duplicateSelectedId:"x",conflictSelectedId:"y",quarantineSelected:new Set(["old"]),
  quarantinePlan:{}, conflictQuarantineSelected:new Set(["old"]),
  conflictQuarantinePlan:{},conflictReviewSelected:new Set(["old"]),
  conflictsSelectedOnly:true
};
const views={manager:{items:[1]},duplicates:{items:[2]},conflicts:{items:[3]}};
const memo={items:[1]};
const conflictMemo={analysis:1};
const context=vm.createContext({
  state, virtualViews:views, managerVisibleMemo:memo,conflictVisibleMemo:conflictMemo,
  t:x=>x,closeDuplicateDetails:()=>{},closeConflictDetails:()=>{},
});
vm.runInContext(js.slice(a,b), context);
vm.runInContext(`reconcileSuccessfulOrganization({
  movedFiles:[{sourcePath:"C:\\\\Mods\\\\Packages\\\\Old\\\\one.package",
    destinationPath:"C:\\\\Mods\\\\Packages\\\\Scripts\\\\Jogabilidade\\\\Author\\\\one.package",
    destinationRelativePath:"Scripts\\\\Jogabilidade\\\\Author\\\\one.package"}]
})`,context);
assert.match(state.items[0].path,/Scripts.*Author/);
assert.match(state.items[0].relativePath,/Jogabilidade/);
assert.equal(state.items[0].id,state.selectedId);
assert.equal(state.items[1].status,"unknown");
assert.equal(state.items[1].relativePath,"Old\\two.package");
assert.equal(state.selectedForPlan.size,0);
assert.equal(state.plan,null);
assert.equal(state.analysisRunId,1);
assert.equal(state.duplicatesAnalysis,null);
assert.equal(state.conflictsAnalysis,null);
assert.equal(state.analysisStatus.duplicates,"not_run");
assert.equal(state.analysisStatus.conflicts,"not_run");
assert.equal(state.quarantineSelected.size,0);
assert.equal(memo.items,null);
assert.equal(views.manager.items,null);
// Real report regression: a Mods - Copia scan contains 375 Unknown entries,
// which must be selectable for safe Not Categorized handling, not hidden.
const eligibleFrom = js.indexOf("function eligibleForPlan(item) {");
const eligibleTo = js.indexOf("function statusLabel(", eligibleFrom);
assert(eligibleFrom >= 0 && eligibleTo > eligibleFrom);
const eligibility = vm.createContext({});
vm.runInContext(js.slice(eligibleFrom,eligibleTo),eligibility);
assert.equal(vm.runInContext(
  'eligibleForPlan({path:"C:/Mods - Copia/Packages/Legacy/unknown.package",status:"unknown"})',
  eligibility), true);
assert.equal(vm.runInContext(
  'eligibleForPlan({path:"x",status:"classified",destinationPath:"Accessories/Headwear",classificationConfidence:"high"})',
  eligibility), true);
assert.equal(vm.runInContext(
  'eligibleForPlan({path:"x",status:"classified",destinationPath:"Scripts/Gameplay/Creator",classificationConfidence:"medium",detectedFrom:["ModFolderCompanion"]})',
  eligibility), true);
assert.equal(vm.runInContext(
  'eligibleForPlan({path:"x",status:"mixed",destinationPath:"Clothing"})',
  eligibility), false);
assert.equal(vm.runInContext(
  'eligibleForPlan({path:"x",status:"invalid"})',
  eligibility), false);
const cfgStart = js.indexOf("function supportsResourceCfgUpdate(folder) {");
const cfgEnd = js.indexOf("function planCanExecute(",cfgStart);
assert(cfgStart>=0 && cfgEnd>cfgStart);
const cfgCtx=vm.createContext({});
vm.runInContext(js.slice(cfgStart,cfgEnd),cfgCtx);
assert.equal(vm.runInContext('supportsResourceCfgUpdate("C:/Sims 3/Mods - Copia")',cfgCtx),true);
assert.equal(vm.runInContext('supportsResourceCfgUpdate("C:/Sims 3/Mods - Copia/Packages/Legacy")',cfgCtx),true);
assert.equal(vm.runInContext('supportsResourceCfgUpdate("C:/Other/CC Incoming")',cfgCtx),false);
assert.match(js,/plannedAction: plannedBySource.get/);
assert.match(js,/planDestination: plannedBySource.get/);
assert.match(js,/state\.analysisRefreshQueued = true/);
assert.match(js,/state\.analysisPipelineBusy = false/);
const planner = read("src-tauri/src/planner.rs");
assert.match(planner,/if item\.status == "unknown" && !source_uses_overrides/);
assert.match(planner,/plan_status: destination_status\.to_string\(\)/);
assert.match(planner,/verify.*companion|verified_companion/);
assert.match(planner,/validate_destination_parts\(&fallback_parts\)/);
assert.match(planner,/fit_destination_to_resource_cfg\(/);
console.log("Organization incremental reconciliation and no-full-rescan: PASS");
