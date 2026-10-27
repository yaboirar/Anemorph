# Changelog

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
