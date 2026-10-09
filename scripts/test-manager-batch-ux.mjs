// Focused headless regression for multi-select and duplicate survivor safety.
// Executed on every portable build before Rust tests.
import assert from "node:assert/strict";
import vm from "node:vm";
import { readFileSync } from "node:fs";

const read = path => readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const source = read("src/app.js");
const html = read("index.html");
const backend = read("src-tauri/src/executor.rs");
const review = read("src-tauri/src/review_store.rs");
const get = (start, end) => {
  const a = source.indexOf(start), b = source.indexOf(end, a);
  assert(a >= 0 && b > a, `Missing functional section: ${start}`);
  return source.slice(a, b);
};
for (const id of [
  "conflicts-mark-intentional-btn", "conflicts-mark-reviewed-btn",
  "conflicts-mark-ignored-btn", "conflicts-clear-marks-btn",
  "duplicates-select-visible-btn",
]) assert(html.includes(`id="${id}"`), `Missing actual control ${id}`);
assert.match(source, /set_conflict_decisions_bulk/);
assert.match(source, /selectConflictReviewRange/);
assert.match(source, /postQuarantineNotice/);
assert.match(backend, /spawn_blocking/);
assert.match(backend, /verify_identity\(&source, expected_hash, item.size\)/);
assert.doesNotMatch(backend, /verify_identity\(&destination, expected_hash, item.size\)/);
assert.match(review, /fn apply_bulk_changes/);
assert.match(review, /fn bulk_validation_is_atomic/);

const groups = [
  { id: "g1", members: [{path:"A"},{path:"B"},{path:"C"}] },
  { id: "g2", members: [{path:"D"},{path:"E"}] },
];
const state = {
  duplicatesAnalysis:{}, quarantineBusy:false, quarantineSelected:new Set(),
  quarantineExactBatch:false, quarantinePlan:null, duplicatesError:"",
  duplicatesNotice:"", lastExactGroupAnchor:null,
};
const context = vm.createContext({
  state,
  exactDuplicateGroups:()=>groups,
  compareExactKeepers:(a,b)=>a.path.localeCompare(b.path),
  visibleDuplicateFindings:()=>[{id:"g1",findingType:"group",kind:"exact_duplicate"}],
  workspaceReadOnly:()=>false,
  t:x=>x, renderDuplicates:()=>{},renderDuplicatesPreview:()=>{},
});
vm.runInContext(
  get("function selectFilteredExactDuplicateGroups() {", "function selectOnlyCurrentExactDuplicateGroup() {"),
  context
);
vm.runInContext("selectExactGroupRows(new Set(['g1']), true)", context);
assert.deepEqual([...state.quarantineSelected].sort(), ["B","C"],
  "Three identical copies must leave one survivor");
vm.runInContext("selectExactGroupRows(new Set(['g2']), true)", context);
assert.deepEqual([...state.quarantineSelected].sort(), ["B","C","E"],
  "Selection accumulates across groups without deleting their survivors");
vm.runInContext("selectExactGroupRows(new Set(['g1']), false)", context);
assert.deepEqual([...state.quarantineSelected].sort(), ["E"],
  "Deselecting one group cannot clear another group");
state.quarantineSelected.clear();
vm.runInContext("selectFilteredExactDuplicateGroups()", context);
assert.deepEqual([...state.quarantineSelected].sort(), ["B","C"],
  "Filtered selection must not include hidden exact groups");
state.quarantineSelected = new Set(["NON_EXACT"]);
vm.runInContext("selectFilteredExactDuplicateGroups()", context);
assert.deepEqual([...state.quarantineSelected], ["NON_EXACT"],
  "Mixed uncertain duplicate selection must not silently become a batch");

const findings = [{id:"first"},{id:"second"},{id:"third"},{id:"fourth"}];
const reviewState = {conflictReviewSelected:new Set(),lastConflictReviewAnchor:"first",conflictsSelectedOnly:false};
const reviewContext = vm.createContext({
  state:reviewState, conflictVisibleMemo:{analysis:"ready"},
  visibleConflictFindings:()=>findings,
  renderConflicts:()=>{},
  renderConflictVirtualRows:()=>{},
  el:{conflictsList:{}},
});
vm.runInContext(
  get("function selectConflictReviewRange(", "function toggleConflictReviewFromRow("),
  reviewContext
);
vm.runInContext("selectConflictReviewRange('third')", reviewContext);
assert.deepEqual([...reviewState.conflictReviewSelected].sort(), ["first","second","third"],
  "Shift range must include every intervening row");
reviewState.lastConflictReviewAnchor="second";
vm.runInContext("selectConflictReviewRange('third', false)", reviewContext);
assert.deepEqual([...reviewState.conflictReviewSelected], ["first"],
  "Range deselection must affect just the requested interval");
// The user intentionally removed "Open Current Folder" from the toolbar.
// Do not resurrect or require it in the regression suite.
assert(!html.includes('id="open-current-folder-btn"'), "Removed folder button must stay removed");
for (const control of ["open-organized-folder-btn"]) {
  assert(html.includes(`id="${control}"`), `Missing working folder navigation: ${control}`);
}
assert(!get("async function executeOrganization() {", "async function chooseManifest() {")
  .includes("await scanFolder(false, true)"),
  "Organizing should not trigger another full DBPF scan");
assert.match(backend, /apply_confirmed_organization_moves/);
assert.match(backend, /moved_paths/);
const folder = "C:\\Mods\\Packages";
const sourcePath = folder + "\\Legacy\\old.package";
const newPath = folder + "\\Scripts\\Jogabilidade\\Autor\\old.package";
const original = {
  id: sourcePath, path: sourcePath, relativePath: "Legacy\\old.package",
  name: "old.package", status: "classified", category: "Scripts",
};
const localState = {
  folder, items: [original], selectedForPlan: new Set([sourcePath]),
  selectedId: sourcePath, plan:{}, analysisRunId:3,
  duplicatesAnalysis: {old:true}, conflictsAnalysis:{old:true},
  duplicatesError:"", conflictsError:"", duplicatesNotice:"",
  conflictsNotice:"", analysisStatus:{duplicates:"completed",conflicts:"completed"},
  conflictReviewSelected: new Set(), conflictsSelectedOnly:false,
  quarantineSelected: new Set(), quarantinePlan:{}, conflictQuarantineSelected:new Set(),
  conflictQuarantinePlan:{}, notice:"",
};
const view = vm.createContext({
  state:localState, virtualViews:{manager:{items:[]}},
  conflictVisibleMemo:{analysis:"old"},
  t:x=>x, closeDuplicateDetails:()=>{}, closeConflictDetails:()=>{},
});
vm.runInContext(
  get("function reflectCompletedOrganization(moves) {", "async function executeOrganization() {"),
  view
);
vm.runInContext("reflectCompletedOrganization([{from:sourcePath,to:newPath}])",Object.assign(view,{sourcePath,newPath}));
assert.equal(localState.items[0].path,newPath);
assert.equal(localState.items[0].relativePath,"Scripts/Jogabilidade/Autor/old.package");
assert(localState.selectedForPlan.has(newPath));
assert.equal(localState.duplicatesAnalysis,null,"Stale duplicate paths cannot remain actionable");
assert.equal(localState.conflictsAnalysis,null,"Stale conflict paths cannot remain actionable");
assert.equal(localState.analysisStatus.duplicates,"not_run");
assert.equal(localState.analysisStatus.conflicts,"not_run");
assert.equal(localState.analysisRunId,4);
console.log("Incremental organizer and folder navigation regressions: PASS");

console.log("Manager batch selection and safe duplicate survivor regressions: PASS");
