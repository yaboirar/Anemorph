<div align="center">

# Changelog

**What's changed in Anemorph.**

[Home](README.md) · [Download](https://github.com/yaboirar/Anemorph/releases/latest) · [User guide](docs/usage.md)

</div>

---

## 0.0.3 - 2026-10-28

### Latest 0.0.3 patch

- Keep Explorer-imported files checked even when no episode or movie is recognized, including after refreshes. Manual checkmark changes are preserved.
- Send new Explorer selections to the running Anemorph window instead of opening a second one, including across normal and Windows package launches.
- Check the handoff between separate processes without desktop control. All 74 automated tests pass; the version remains 0.0.3.

### Updated 0.0.3 downloads

- Reduce memory use by drawing the window without starting a graphics engine by default.
- Check active use as well as idle: scrolling, large folders, and a 300-file rename followed by undo. All 300 original filenames and file contents were verified after undo.
- Keep files selected when opened from File Explorer, including files whose names do not need changing. Only files that need a rename are processed.
- Show Anemorph as the app name in Task Manager.
- Stop navigation animations from keeping the app busy after the pointer stops moving.
- Pass 72 automated tests. Add support for verified release signing; the current downloads remain unsigned and Windows may still show a warning.

### Fixes

- Remove the repeated Rename with Anemorph option from File Explorer, including after upgrades.
- Stop forcing File Explorer to close during uninstall. Remove Anemorph's own files and settings while preserving your videos and unrelated files.
- Fix the right-click menu setting getting stuck on Working after reinstall.
- Add Repair Anemorph in Settings to reload saved settings, check the folder again, and restore the right-click option. Put Repair and Uninstall together.
- Let users include and exclude files even when their names do not need changing. Those files are not renamed unnecessarily.

### Easier to use

- Add clearer Home buttons and visible separators between right-click choices.
- Move the file and selection counts beside refresh and filter.
- Move the app name, version, and About shortcut to the bottom left.
- Keep the top buttons as icons when animations are turned off.
- Put highlight color inside Appearance, with Language between Appearance and Integration.
- Add Russian, English (US/UK), Spanish, Portuguese, Turkish, German, Japanese, Korean, Simplified Chinese, and Modern Standard Arabic.
- Give episode and movie naming their own help lists and reject options that belong to the other type.

### Speed and checks

- Avoid loading fonts for languages that are not in use.
- Test the app with only two CPU threads available and without using a dedicated graphics card.
- Check large folders, repeated previews, idle memory use, and recovery after interrupted renames.
- Pass 70 automated tests and add screenshots to the README.

See [test results and known limits](docs/performance.md) for the full measurements and technical details.

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
