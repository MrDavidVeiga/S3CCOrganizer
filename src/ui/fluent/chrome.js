/* Accessible NavigationView tab keyboard model.
 * The existing application is still the single source of truth for page
 * selection: we dispatch native button clicks instead of setting app state.
 */
const tablist=document.querySelector(".topbar-mode-switcher[role='tablist']");
function visibleTabs() {
  if (!tablist) return [];
  return [...tablist.querySelectorAll(".mode-btn[data-tab]")]
    .filter(b=>!b.disabled && !b.hidden && !b.classList.contains("hidden"));
}
function syncTabNavigation() {
  if (!tablist) return;
  for(const button of tablist.querySelectorAll(".mode-btn[data-tab]")) {
    const id=button.dataset.tab;
    if (!button.id) button.id=`fluent-nav-${id}`;
    button.setAttribute("role","tab");
    button.setAttribute("aria-controls",`page-${id}`);
    button.tabIndex=button.getAttribute("aria-selected")==="true" && !button.classList.contains("hidden")?0:-1;
    // Resolve against the owning document: observers can still deliver a
    // queued mutation while a JSDOM/WebView document is being torn down.
    const panel=tablist.ownerDocument.getElementById(`page-${id}`);
    if(panel){
      panel.setAttribute("role","tabpanel");
      panel.setAttribute("aria-labelledby",button.id);
    }
  }
  const visible=visibleTabs();
  if(visible.length && !visible.some(b=>b.tabIndex===0)) visible[0].tabIndex=0;
}
if(tablist) {
  syncTabNavigation();
  const observer=new MutationObserver(syncTabNavigation);
  observer.observe(tablist,{subtree:true,attributes:true,attributeFilter:["class","aria-selected","disabled","hidden"],childList:true});
  tablist.addEventListener("keydown",event=>{
    if(event.defaultPrevented||event.altKey||event.ctrlKey||event.metaKey) return;
    if(!["ArrowLeft","ArrowRight","Home","End"].includes(event.key))return;
    const target=event.target.closest?.(".mode-btn[data-tab]");
    const tabs=visibleTabs();
    if(!target || !tabs.includes(target))return;
    event.preventDefault();event.stopPropagation();
    let next=tabs.indexOf(target);
    if(event.key==="Home")next=0;
    else if(event.key==="End")next=tabs.length-1;
    else if(event.key==="ArrowRight")next=(next+1)%tabs.length;
    else next=(next-1+tabs.length)%tabs.length;
    tabs[next].click();
    tabs[next].focus({preventScroll:true});
  });
}
export { syncTabNavigation };
