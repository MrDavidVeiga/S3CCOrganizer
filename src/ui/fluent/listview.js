/**
 * Accessible keyboard navigation for the three virtualized analysis lists.
 * Retains existing selection/click semantics: navigation MOVES FOCUS ONLY.
 * Press Enter/Space to activate a row as before. Does not touch file actions.
 *
 * The scanner's native list renderer owns data-fluent-index and fluentCount;
 * focus requests survive DOM replacement via MutationObserver.
 */
const VIEWS = [
  {id: "package-list", selector: ".package-row", stride: 66, rowHeight: 56, name: "Manager packages"},
  {id: "duplicates-list", selector: ".duplicate-row", stride: 62, rowHeight: 53, name: "Duplicate findings"},
  {id: "conflicts-list", selector: ".conflict-row", stride: 62, rowHeight: 53, name: "Conflict findings"},
];

function setupVirtualList(config) {
  const list = document.getElementById(config.id);
  if (!list || list.dataset.fluentKeyboard === "true") return;
  list.dataset.fluentKeyboard = "true";
  list.setAttribute("aria-label", config.name);
  let awaiting = -1;
  let request = 0;

  function getRow(index) {
    return list.querySelector(`${config.selector}[data-fluent-index="${index}"]`);
  }

  function finishPendingFocus() {
    if (awaiting < 0) return false;
    const row = getRow(awaiting);
    if (!row) return false;
    awaiting = -1;
    row.focus({preventScroll: true});
    return true;
  }

  const mutation = new MutationObserver(() => {
    if (awaiting >= 0) finishPendingFocus();
  });
  mutation.observe(list, {childList: true});

  function scheduleFocus(index) {
    awaiting = index;
    if (finishPendingFocus()) return;
    if (request) cancelAnimationFrame(request);
    // app.js re-renders virtual rows on scroll in requestAnimationFrame.
    let retries = 0;
    const retry = () => {
      request = 0;
      if (finishPendingFocus() || ++retries >= 5) {
        awaiting = -1;
        return;
      }
      request = requestAnimationFrame(retry);
    };
    request = requestAnimationFrame(retry);
  }

  function scrollFor(index) {
    const padding = 8;
    const top = padding + index * config.stride;
    const bottom = top + config.rowHeight;
    const viewport = Math.max(list.clientHeight || 0, config.rowHeight + 1);
    if (top < list.scrollTop + padding) {
      list.scrollTop = Math.max(0, top - padding);
    } else if (bottom > list.scrollTop + viewport - padding) {
      list.scrollTop = Math.max(0, bottom - viewport + padding);
    }
    // Dispatch an explicit scroll event to support keyboard scroll in
    // embedded WebViews and environments with synthetic scrollTop updates.
    list.dispatchEvent(new Event("scroll", {bubbles: false}));
  }

  list.addEventListener("keydown", event => {
    if (event.defaultPrevented || event.altKey || event.ctrlKey || event.metaKey) return;
    if (!["ArrowUp", "ArrowDown", "Home", "End", "PageUp", "PageDown"].includes(event.key)) return;

    const target = event.target;
    if (!(target instanceof Element)) return;
    // Checkboxes are independent, focusable input controls.
    if (target.closest("input,select,textarea,[contenteditable='true']")) return;
    const row = target.closest(config.selector);
    if (!row || !list.contains(row)) return;

    const current = Number(row.dataset.fluentIndex);
    const count = Number(list.dataset.fluentCount || 0);
    if (!Number.isInteger(current) || current < 0 || !Number.isInteger(count) || count < 1) return;
    let next = current;
    const page = Math.max(1, Math.floor((list.clientHeight || 420) / config.stride) - 1);
    switch (event.key) {
      case "ArrowUp": next--; break;
      case "ArrowDown": next++; break;
      case "Home": next = 0; break;
      case "End": next = count - 1; break;
      case "PageUp": next -= page; break;
      case "PageDown": next += page; break;
    }
    next = Math.max(0, Math.min(count - 1, next));
    event.preventDefault();
    event.stopPropagation();
    if (next === current) return;
    scrollFor(next);
    scheduleFocus(next);
  });
}

for (const view of VIEWS) setupVirtualList(view);
export { setupVirtualList };
