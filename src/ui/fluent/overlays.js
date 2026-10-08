/**
 * Shared ContentDialog keyboard and focus behavior.
 * Compatible with the existing modal open/close functions: observe class
 * changes instead of replacing application handlers or mutating file actions.
 */
import { closeFluentComboBox } from "./combobox.js";

const modalIds=[
  "plan-modal","duplicate-details-modal","conflict-details-modal",
  "collision-review-modal","structure-modal","confirm-modal",
  "catalog-import-modal"
];
const closeButtons={
  "plan-modal":"plan-close-btn",
  "duplicate-details-modal":"duplicate-details-close-btn",
  "conflict-details-modal":"conflict-details-close-btn",
  "collision-review-modal":"collision-review-close-btn",
  "structure-modal":"structure-modal-cancel-btn",
  "confirm-modal":"confirm-cancel-btn",
  "catalog-import-modal":"catalog-import-cancel"
};
const shell=document.querySelector(".app-shell");
const modals=modalIds.map(id=>document.getElementById(id)).filter(Boolean);
const stack=[];

const focusSelector=[
  'button:not([disabled])','[href]:not([aria-disabled="true"])',
  'input:not([disabled]):not([type="hidden"])','select:not([disabled])',
  'textarea:not([disabled])','[tabindex]:not([tabindex="-1"])'
].join(",");

function visible(element){
  return element.isConnected && !element.closest(".hidden,[hidden],[inert]") &&
    element.getClientRects().length > 0;
}
function focusables(container){
  return Array.from(container.querySelectorAll(focusSelector)).filter(e=>
    visible(e) && e.tabIndex >= 0 && e.getAttribute("aria-hidden") !== "true"
  );
}
function focusFirst(container){
  if(!container) return;
  const dialog=container.querySelector('[role="dialog"]');
  const items=focusables(container);
  const prefer=items.find(x=>x.matches('input:not([readonly]):not([type="checkbox"])'));
  const chosen=prefer||items[0]||dialog;
  if(chosen===dialog && !dialog.hasAttribute("tabindex"))dialog.setAttribute("tabindex","-1");
  chosen?.focus({preventScroll:true});
}
function isOpen(modal){return !modal.classList.contains("hidden");}
function current(){return stack.at(-1) || null;}

function activateTop(){
  const top=current()?.modal || null;
  if(shell) shell.inert=!!top;
  for(const modal of modals){
    modal.inert=!!top && modal!==top;
  }
}

function restoreFocus(entry){
  const candidate=entry?.returnTarget;
  if(candidate && visible(candidate) && !candidate.hasAttribute("disabled")){
    candidate.focus({preventScroll:true});
    return;
  }
  const top=current();
  if(top){focusFirst(top.modal);return;}
  // Virtualized rows may have been removed during a review. Focus the
  // current page's selected tab instead of a detached DOM node.
  const selected=document.querySelector('.tabs .mode-btn.active:not([disabled])');
  if(selected && visible(selected))selected.focus({preventScroll:true});
}

function syncModals(){
  const wasTop=current();
  let removedTop=null;
  for(let i=stack.length-1;i>=0;i--){
    if(!isOpen(stack[i].modal)){
      if(wasTop===stack[i])removedTop=stack[i];
      stack.splice(i,1);
    }
  }
  const added=[];
  for(const modal of modals){
    if(isOpen(modal) && !stack.some(e=>e.modal===modal)){
      const entry={modal,returnTarget:document.activeElement};
      stack.push(entry);added.push(entry);
      modal.setAttribute("aria-hidden","false");
    }
  }
  stack.forEach((entry,index)=>{entry.modal.style.zIndex=String(1100+10*index);});
  for(const modal of modals){
    if(!isOpen(modal)){
      modal.style.zIndex="";
      modal.inert=false;
      modal.setAttribute("aria-hidden","true");
    }
  }
  const top=current();
  if(top!==wasTop){
    closeFluentComboBox();
    activateTop();
    if(top){
      focusFirst(top.modal);
    }else if(removedTop){
      restoreFocus(removedTop);
    }
  }else activateTop();
}

for(const modal of modals){
  new MutationObserver(syncModals).observe(modal,{attributes:true,attributeFilter:["class"]});
}
syncModals();

document.addEventListener("keydown",event=>{
  const top=current();
  if(!top)return;
  if(event.key==="Escape"){
    // The ComboBox module intercepts Escape first when its listbox is open.
    event.preventDefault();
    event.stopImmediatePropagation();
    const closeButton=document.getElementById(closeButtons[top.modal.id]);
    if(closeButton && !closeButton.disabled)closeButton.click();
    return;
  }
  if(event.key!=="Tab")return;
  const items=focusables(top.modal);
  if(!items.length){event.preventDefault();focusFirst(top.modal);return;}
  const first=items[0], last=items[items.length-1];
  const active=document.activeElement;
  if(!top.modal.contains(active)){
    event.preventDefault();
    (event.shiftKey?last:first).focus({preventScroll:true});
  } else if(event.shiftKey && active===first){
    event.preventDefault();last.focus({preventScroll:true});
  } else if(!event.shiftKey && active===last){
    event.preventDefault();first.focus({preventScroll:true});
  }
},true);

document.addEventListener("focusin",event=>{
  const top=current();
  if(!top || top.modal.contains(event.target))return;
  focusFirst(top.modal);
},true);

export function activeFluentDialogId(){return current()?.modal?.id || null;}
