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
