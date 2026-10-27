# Changelog

## 0.0.3 - 2026-10-28

### Explorer integration and installation

- Fix duplicate Rename with Anemorph entries by making modern and classic registrations mutually exclusive, including stale COM registrations during upgrades.
- Run both menu modes in Anemorph's own broker instead of loading a DLL into Explorer.
- Remove forced Explorer shutdown/restart from uninstall. Close only Anemorph's host and remove app-owned files, settings, shortcuts, and registration while preserving unrelated files.
- Keep legacy DLL removal safe when Windows still holds an older file open; report the lock instead of freezing the desktop.
- Fix Integration remaining on Working after reinstall. Status checks are read-only, helper operations have timeouts, and the resident host inherits no parent handles.
- Add Repair Anemorph in Settings to reload saved preferences, rescan files, and repair Explorer registration. Group Repair and Uninstall in one panel.

### Everyday use

- Let unchanged files be included or excluded without turning them into rename operations.
- Add clear separation between context-menu choices and visible surfaces to Home actions.
- Move file and selection counts beside refresh/filter controls.
- Move the app name, version, and About shortcut to the bottom-left footer.
- Keep navigation icon-only when animations are disabled.
- Merge highlight color into Appearance and place Language between Appearance and Integration.
- Add Russian, English (US/UK), Spanish, Portuguese, Turkish, German, Japanese, Korean, Simplified Chinese, and Modern Standard Arabic interfaces. Load script fonts on demand and shape Arabic text.
- Separate episode/movie format reference lists and reject indexes belonging to the other format.

### Performance and verification

- Prefer integrated graphics and retain CPU-rendering fallback, without loading CJK fonts for English users.
- Add repeatable constrained benchmarks, software-rendering checks, and repeated-preview private-memory sampling.
- Validate on two logical processors using Microsoft's CPU renderer; publish the measurements and limitations in docs/performance.md.
- Add regression coverage for integration mode changes, helper timeouts, unchanged-file selection, format separation, translations, and font coverage.
- Include Home and Explorer menu screenshots in the README.

## 0.0.2 - 2026-10-27

### Movies and filename formats

- Support 26 video containers, with individual filter choices.
- Detect movie markers and add movie formats using `[Movie Name]`, `[Movie]`, and `[Space]`.
- Rename Episode format to Filename format, with episode and movie editors in one panel.
- Add persistent right-click classification as movie or episode, including manual numbers for unrecognized episodes.
- Add creator credit linking to @yaboirar on GitHub in About.

### Preview and filtering

- Preserve recognized episode numbers and gaps with automatic detection on or off. Sequential numbering is a separate opt-in for unrecognized files.
- Recognize separated season/episode pairs such as `S3 - 23`.
- Add draggable filename and status column widths, consistent status alignment, and corrected resize handles.
- Show No change, Title changed, and Done states for their corresponding results.
- Restore the detected series name through its refresh button.
- Allow multiple container filters, bulk selection, and Shift-click ranges. No active filter shows all supported containers.
- Separate visible Home filter choices from active filtering. Default visible choices are MKV, MP4, MOV, AVI, WebM, and M4V.
- Refine About spacing, navigation colors, and Integration controls.

### Windows installation and integration

- Add an installer with destination selection, Installed apps registration, Start menu shortcuts, repair, and uninstall.
- Integrate Rename with Anemorph into Explorer for supported video selections, retaining that selection through filtering, refresh, rename, and undo.
- Add optional Windows 11 first-menu integration with certificate approval and classic-menu fallback.
- Add in-app Integration controls and an uninstall action in Settings.
- Remove Anemorph-owned data, registration, shortcuts, recorded installation leftovers, and exact app cache entries during uninstall.
- Detect menu DLL locks before deleting app files or settings and offer an Explorer restart to finish cleanup.
- Keep same-folder repair working while Explorer holds an older menu DLL.

## 0.0.1  -  2026-10-26

Initial versioned Anemorph build.

- Native Windows interface with dark/light themes, custom highlight colors, and zoom.
- Automatic season/episode detection, editable naming formats, and container filters.
- Preview, range selection, collision protection, undo, and interrupted-batch recovery.
- Background scanning and large previews, virtualized rows, and reduced renderer memory allocation.
- Version displayed beneath the app name and in About; Windows executable metadata uses the same version.
