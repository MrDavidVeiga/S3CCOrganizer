# WinUI 3 visual fidelity — implementation log

Development branch: `feature/winui3-full-fidelity`. No change to the public `main` or the package-analysis engine.

## Batch 1 — foundations and basic controls

- Add `src/ui/fluent/tokens.css` as semantic aliases over the existing theme system (`src/theme.js` and `src/winui.css`).
- Add `src/ui/fluent/controls.css` to normalize buttons, text/search inputs, checked/unchecked/indeterminate checkboxes, keyboard focus, disabled/hover/pressed behavior.
- Keep all seven themes, existing IDs, Tauri commands, virtual lists, and handlers intact.
- Load the new foundation *after* the two legacy stylesheets to allow safe, incremental migration.
- Align control radii with Windows geometry guidance (4px controls, 8px overlays).
- Add static regression verification and branch-only frontend-build validation.

### Boundaries

The native `<select>` popup remains platform-rendered; only its closed field gets basic TextBox styling in this batch.
There is **no claim** yet of complete ComboBox, SearchBox, MenuFlyout, ListView or ContentDialog parity.
The titlebar has not yet gained native Windows Mica/Snap integration.
Focusable modal trapping and return-focus are for the overlays batch.
Dynamic action buttons and checkboxes inherit the global basic-control styles without rewriting `src/app.js`.

## Validation

Run:

```bash
npm install
node scripts/verify-fluent-foundation.mjs
npm run build
```

On this development branch, GitHub Actions `Validate Fluent foundation` runs the same checks.
Afterwards, visual QA must use the actual Tauri WebView on Windows and cross-platform builds,
covering normal/hover/pressed/focus/disabled/checked/indeterminate modes and all themes.

## Next

1. Improve full TextBox/SearchBox semantics and implement the keyboard-correct ComboBox dropdown component.
2. Add reusable modal focus management and bring all seven ContentDialogs/flyouts onto it.
3. Complete navigation/collections/scrollbars/InfoBar/progress and theme visual regression.
4. Only merge into `main` after runtime verification.

Source: https://learn.microsoft.com/en-us/windows/apps/design/signature-experiences/geometry

## Batch 2A — ComboBox migration

The thirteen static `<select>` controls are enhanced by `src/ui/fluent/combobox.js` with a custom WinUI-styled button and a shared, virtualized listbox. The original select remains the source of truth and still emits normal `input`/`change` events; existing Manager, Duplicates, Conflicts, Tools, Restore and Catalog logic is not rewritten. Supports mouse, arrows, Home/End, Enter/Space, Escape, Tab, and incremental keyboard typeahead. Dynamic options refresh via MutationObserver. Popup uses a fixed overlay outside scroll containers and short visible row windows for large folders.

**Limitations pending runtime verification:** active option markup and popup positioning should be tested inside the real Tauri WebView, especially when options are repopulated and with screen readers.

## Batch 2B — ContentDialog + flyout focus behavior

Added `src/ui/fluent/overlays.js` and `overlays.css` to standardize keyboard focus trapping and restoration, nested modal stacking, Escape cancellation using existing close buttons, background inertness, and dialog visual tokens. All seven existing modal IDs and actions remain owned by app/catalog code. Added `menus.js` for Arrow/Home/End/Enter/Escape keyboard handling and role/selection synchronization on the Language/Status/Theme menus. Fixed ComboBox initial active scroll. Catalog import dialog now has an explicit labelledby target.

**Still requires:** Browser/WebView runtime tests of modal nesting and focus return when a virtualized list item is replaced. No application-logic backend change, no merge to main.

## Interaction simulator (Batch 2)

`scripts/test-fluent-interactions.mjs` uses JSDOM in the branch-only Windows CI workflow to verify keyboard/pointer ComboBox selection with native select event propagation, dynamic option refresh, 1200-entry virtualization, language-menu keyboard navigation, and nested modal Escape/focus restoration. This is not a screenshot/WebView accessibility test; that remains a separate release gate.

## UI feedback patch — Oct 8

- ComboBoxes: pointer hover now updates the highlighted option without replacing the hovered DOM element.
- Closed ComboBox: Up/Down selects the previous/next enabled option immediately (without opening); native input/change events remain intact. Alt+Down opens; Enter/Space opens.
- EA App theme: accent #276AFC, hover #3978FC, pressed #215BD8, with white text and matching theme swatch.
- Lists: hover, outline and selected state are centered vertically with label/count, keeping virtual row heights 56px (Manager) and 53px (Duplicates/Conflicts), with existing gaps.
- Automated interaction and theme regression checks to follow in the same branch.

## Batch 3 — ListView navigation and ScrollBar fidelity

- Merge the latest Restore / quarantine / confirmation UI branch before beginning.
- Keep the **approved row visuals**, existing row heights and virtual scroll strides.
- Attach stable item indices and current filtered-item count in all three virtual renderers.
- `listview.js`: keyboard Arrow Up/Down, Home, End, Page Up/Down moves **focus only**; Enter/Space retain their existing selection/dialog behaviour; checkbox keys remain separate. Focus survives virtual-row reconstruction on scroll.
- `scrollbars.css`: maintain the native Windows/WebView2 scroll mechanics, themed thinner thumbs at rest, wider paint on hover/pressed while keeping a 12px channel to avoid content-width jitter, light/dark/high-contrast variants.
- Test focus navigation at virtual boundaries and ensure list performance for large packages. No scanner/Rust behavior is modified.

The visual thumb behavior requires checking in an actual Tauri build: OS and WebView versions can render native overlay scrollbars differently.

## Batch 4 — ProgressBar / ProgressRing / InfoBar

- Synced the newly created Sims3Pack content-aware extraction branch before implementation; neither main nor the public Beta is changed.
- Three existing Manager / Duplicates / Conflicts operation tracks now expose `role=progressbar` with accurate `aria-valuenow` when total is known and no numeric value for indeterminate work.
- Determinate ProgressBar uses a compact Fluent 4px accent rail, and unknown totals use a continuously travelling indeterminate indicator.
- Existing busy states use a ProgressRing visual in the new compact InfoBar design; no independent spinner polling was added.
- Semantic non-dismissable InfoBar treatment for existing `.scan-state` messages (info, busy, success, warning and error). Severity comes from explicit state classes or structured scan statistics, not from translated message content.
- Supports all seven themes, reduced motion and forced-colors. Never presents the current package filename in progress status.
- Regression tests cover accessible percentages, unknown totals, cancellation, warning and error announcements, and the existing privacy restriction.

WebView2 appearance still needs visual acceptance with the new Windows x64 diagnostic build.

## Combined batches 5–6 (Oct 8)

Batch 5: native details are Fluent Expanders, accessible search inputs are SearchBoxes with a clear button that dispatches existing input/change events, and native tooltips are progressively themed. Batch 6: caption glyphs are vector-normalized; NavigationView keyboard focus/selection and Fluent animations are refined. Respect theme choices and all existing backend operations.

### Stage 5 + 6 validation
- Dedicated DOM tests for SearchBox's localized clear affordance and original input/change dispatch, native Expander state, keyboard navigation skipping hidden tabs, vector caption icon geometry and asynchronous maximize state, and mouse/keyboard ToolTip accessibility.
- The default window's Tauri menu behavior still belongs to `src/theme.js` / `getCurrentWindow()`; avoid duplicate resize/window click handlers.
- Build is diagnostic Windows x64 only, without release/tag/merge.
