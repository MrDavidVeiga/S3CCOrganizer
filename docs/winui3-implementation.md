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

### Mean Girls primary action parity

Following the campaign's **Get Kit** CTA, primary action buttons in Mean Girls are near-black `#101010` with white `#FFFFFF` text (hover `#292929`, pressed `#050505`). Pink `#F92F60` remains the selection/progress/navigation accent. Standard EA App primary actions remain blue `#276AFC` with white text. This is a theme-specific CSS override, not a shared accent token, so all seven themes retain their other surfaces and semantic status colors.

## Stage 7 — WinUI 3 Button geometry audit

- The app-wide Primary, Secondary and Danger commands share a **32px minimum** control height, **14px Segoe UI Variable Semibold (600)**, **20px** line-height, **5px vertical / 12px horizontal** padding, **1px borders** and Fluent control radius (4px by default).
- The inherited legacy CSS previously gave Primary 12px vertical padding while Secondary used 9px; Remove Record and Refresh had the same font family but inconsistent dimensions. The new last-loaded `button-geometry.css` unifies actual geometry and icon alignment without changing existing handlers, responsive widths or theme colors.
- Exception: Choose Mods Folder / Scan CCs in the left sidebar intentionally remain **large, 52px minimum**, and utility commands may use a compact 12px font to avoid overflow.
- Buttons are allowed to grow to accommodate wrapped translations; never clip accessible labels simply to force a fixed pixel height.
- Primary styling retains Mean Girls black/white and EA App blue/white. Success, warning and error colors remain semantic and theme-independent. Focus/hover/pressed behaviors remain under their original control styles.

### Manager selection toolbar — Portuguese and Spanish wrapping
The Manager's **Select All / Select None** pair is kept on one line in translations such as **Selecionar Tudo / Selecionar Nenhum** without reducing the standard 14px Fluent label. Compact 7px horizontal padding inside this preview-column toolbar is deliberate. Below 1180px the two controls occupy a separate right-aligned row rather than shrinking the search field; at 520px or smaller the pair may fill the available full row. All selection handlers and original i18n keys stay unchanged.

### Converter toolbar — keep full PT/ES labels on one line

The converter's old fixed 190px flex basis made the primary **Converter para .package** grow to two lines in Brazilian Portuguese. All three actions (select Sims3Pack, select destination, convert) now share a 240px basis with `white-space: nowrap`, normal 14px Semibold and unchanged labels/icons. With limited horizontal space the flex toolbar wraps **whole buttons**, and under 620px stacks them at full available width. EA App blue, Mean Girls black/white, original handlers and conversion logic are preserved. Verify at 100%, 125%, and 150% scaling in the Windows build.

### WinUI 3 path contrast — light themes
The converter previously used a fixed light foreground (#d4dde1) for filesystem paths, making them nearly unreadable on Mean Girls, Veiga Light, WinUI Light and System Light. The last-loaded tool-paths.css now uses semantic Fluent foregrounds, WinUI card surfaces and contrast-aware strokes. Both source and output paths remain complete; results provide full-path hover details and more legible 11px monospaced text. Regression assertions run in the Windows CI pipeline.

## Seven-theme final consistency audit — hover, surfaces, text and status

**Baseline:** feature/winui3-full-fidelity after the previous Stage 7 button sizing, full language labels, Sims3Pack content-aware fixes and light-theme path correction. The main branch stays unchanged.

### Confirmed code findings
1. The light theme's `--winui-control-fill-hover: rgba(249,249,249,.50)` was nearly white and inherited by several ListViews, ComboBox popup options and menus. On white panels, it made hover wash out or disappear.
2. The Manager/ Duplicates/ Conflicts lists, Restore history, Structure and Catalog had diverged hover, active and selected rules. Some drew the same color for hover and selected, reducing the visual distinction.
3. The catalog's fixed dark header/footer `#171d22` and multiple translucent-white row/card separators were designed for dark backgrounds but remained in light themes.
4. Several status foregrounds such as `#cfe5c3` (success), `#f2b7b7` (error) and `#e8c38a` (warning) lacked contrast in the light themes.
5. File paths in Manager plans, duplicates, conflicts, restore and technical metadata still used light-on-dark constants even after the Sims3Pack path fix.
6. The EA App's blue-on-dark ListView fill and white active indicator, Mean Girls' pink accent and black/white CTA, and green success InfoBar must remain intentional exceptions.

### Remediation
- Final-loaded `src/ui/fluent/theme-audit.css` uses semantic hover/pressed/selected tokens by resolved mode, plus Mean Girls and EA App exceptions.
- Stronger but restrained, darker-on-light hover for Manager, Duplicates, Conflicts, Restore, Structure, Catalog, menus and the virtual ComboBox options; selection always remains distinguishable. No row padding, height, item count or virtual scrolling logic is changed.
- Theme-aware Catalog headers, borders and pane backgrounds; all system/Veiga/WinUI light and dark colors resolve via existing tokens.
- Semantic green/amber/red/blue status text uses the existing InfoBar `--fluent-state-*` values for contrast across palettes, including Mean Girls.
- Correct formerly pale technical/filepath text in all workflow dialogs and secondary panels.
- Preserve native focus and forced-colors accessibility; no functional JS/Rust modifications.

### Validation boundary
Automated CI can validate CSS wiring, state declarations, theme coverage, statuses, contrast examples and frontend/Rust builds. Only a **Windows WebView2 visual test** can establish final pixel-level hover/focus fidelity at 100%, 125% and 150% scaling in the seven themes. That manual acceptance remains necessary before merging to `main`.

### Final integration checkpoint
The current WinUI branch includes the latest completed-quarantine history removal in Tools (frontend/Rust with explicit confirmation and recovered-state safeguards) and the latest main scanner improvements. The audit still uses semantic theme tokens rather than replacing these features. Windows UI-test CI now includes a seven-theme hover/selection walkthrough in the artifact README and cancels outdated runs for the same development branch.

### Seven-theme finishing audit — stronger light hover and secondary list coverage
Raised light-mode neutral ListView hover from 7.5% to 10.5% black and pressed to 17% while keeping selected accent fill. Mean Girls uses dark-plum hover at 15% with 21% pressed; theme identity and black primary actions are preserved. Completed Tools, Restore, Catalog, virtual detail rows, nested actions, disabled and keyboard focus rules. No change to row heights, virtual scroll, item selection mechanics or scanner logic. Static/WCAG test assertions were expanded; real WebView2 inspection is still required for each theme, focus, hover and DPI.
