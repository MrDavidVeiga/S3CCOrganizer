/**
 * Accessible keyboard support for the app's existing Language, Status and
 * Theme flyouts. Existing click handlers remain responsible for state changes.
 */
const components=[
  {button:"language-button",menu:"language-menu",kind:"listbox"},
  {button:"status-filter-toggle-btn",menu:"status-filter-menu",kind:"listbox"},
  {button:"theme-button",menu:"theme-flyout",kind:"menu"}
];

for(const entry of components){
  const trigger=document.getElementById(entry.button);
  const menu=document.getElementById(entry.menu);
  if(!trigger || !menu)continue;
  trigger.setAttribute("aria-controls",menu.id);
  if(entry.kind==="listbox")menu.setAttribute("aria-label",trigger.getAttribute("aria-label") || "Options");

  const options=()=>Array.from(menu.querySelectorAll(entry.kind==="menu"?'[role="menuitemradio"]':'button'));
  const isOpen=()=>!menu.classList.contains("hidden");
  function refresh(){
    if(entry.kind==="listbox"){
      for(const opt of options()){
        opt.setAttribute("role","option");
        opt.tabIndex=-1;
        const selected=entry.menu==="language-menu"
          ? opt.dataset.lang===document.documentElement.lang
          : opt.classList.contains("active");
        opt.setAttribute("aria-selected",String(selected));
      }
    }else{
      for(const opt of options())opt.tabIndex=-1;
    }
  }
  function focusOption(which="selected"){
    refresh();
    const enabled=options().filter(opt=>!opt.disabled);
    const selected=enabled.find(opt=>opt.getAttribute(entry.kind==="menu"?"aria-checked":"aria-selected")==="true");
    const next=which==="last"?enabled.at(-1):which==="first"?enabled[0]:selected||enabled[0];
    next?.focus({preventScroll:true});
  }
  function move(delta){
    const candidates=options().filter(el=>!el.disabled);
    if(!candidates.length)return;
    const at=candidates.indexOf(document.activeElement);
    // Normalize index after addition to support wrapping in both directions.
    const index=at<0?(delta>0?0:candidates.length-1):(at+delta+candidates.length)%candidates.length;
    candidates[index]?.focus({preventScroll:true});
  }
  trigger.addEventListener("keydown",event=>{
    if(!["ArrowDown","ArrowUp","Enter"," "].includes(event.key))return;
    event.preventDefault();event.stopPropagation();
    const wasOpen=isOpen();
    if(!wasOpen)trigger.click();
    queueMicrotask(()=>{
      if(isOpen())focusOption(event.key==="ArrowUp"?"last":"selected");
    });
  });
  menu.addEventListener("keydown",event=>{
    if(event.key==="Escape"){
      event.preventDefault();event.stopPropagation();
      if(isOpen())trigger.click();
      trigger.focus({preventScroll:true});
    }else if(event.key==="ArrowDown" || event.key==="ArrowUp"){
      event.preventDefault();event.stopPropagation();
      move(event.key==="ArrowDown"?1:-1);
    }else if(event.key==="Home" || event.key==="End"){
      event.preventDefault();
      focusOption(event.key==="Home"?"first":"last");
    }else if((event.key==="Enter" || event.key===" ") && options().includes(document.activeElement)){
      event.preventDefault();
      document.activeElement.click();
    }else if(event.key==="Tab"){
      if(isOpen())trigger.click();
    }
  });
  menu.addEventListener("click",()=>{
    queueMicrotask(()=>{if(!isOpen())trigger.focus({preventScroll:true});});
  });
  new MutationObserver(refresh).observe(menu,{childList:true,subtree:true,attributes:false});
  refresh();
}
