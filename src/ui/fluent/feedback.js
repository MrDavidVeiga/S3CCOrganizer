/**
 * Progressive InfoBar enhancement.
 * The original .scan-state text and classes remain the sole message source.
 * Structured warnings may be indicated via data-fluent-severity="warning";
 * never guess severity from translated text.
 */
const statusNodes = Array.from(document.querySelectorAll(".scan-state"));
const toneOf = el => {
  if (el.classList.contains("error")) return "error";
  if (el.classList.contains("busy")) return "busy";
  if (el.dataset.fluentSeverity === "warning") return "warning";
  if (el.classList.contains("success")) return "success";
  return "info";
};

function syncFluentInfoBar(el) {
  if (!el.classList.contains("fluent-infobar")) el.classList.add("fluent-infobar");
  const message = el.textContent.trim();
  if (!message) {
    el.removeAttribute("role");
    el.removeAttribute("aria-live");
    el.removeAttribute("aria-busy");
    el.removeAttribute("data-fluent-tone");
    return;
  }
  const tone = toneOf(el);
  if (el.dataset.fluentTone !== tone) el.dataset.fluentTone = tone;
  const role = tone === "error" ? "alert" : "status";
  if (el.getAttribute("role") !== role) el.setAttribute("role", role);
  const live = tone === "error" ? "assertive" : "polite";
  if (el.getAttribute("aria-live") !== live) el.setAttribute("aria-live", live);
  if (el.getAttribute("aria-atomic") !== "true") el.setAttribute("aria-atomic", "true");
  const busy = tone === "busy" ? "true" : "false";
  if (el.getAttribute("aria-busy") !== busy) el.setAttribute("aria-busy", busy);
}

for (const el of statusNodes) {
  const observer = new MutationObserver(() => syncFluentInfoBar(el));
  observer.observe(el, {
    attributes: true,
    attributeFilter: ["class", "data-fluent-severity"],
    childList: true,
    characterData: true,
    subtree: true,
  });
  syncFluentInfoBar(el);
}
export { syncFluentInfoBar };
