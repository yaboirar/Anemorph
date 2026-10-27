# Anemorph

**Rename a video library without losing its episode numbers.**

Anemorph is a free, open-source Windows app for batch-renaming anime, TV episodes, and movies. Review every proposed name before applying a batch, then undo the last batch if you need to. Video contents stay untouched.

[**Download v0.0.3**](https://github.com/yaboirar/Anemorph/releases/tag/v0.0.3) · [What's changed](CHANGELOG.md#003---2026-10-28) · [User guide](docs/usage.md)

![Anemorph showing original names, proposed names, and unchanged files](assets/screenshots/home.jpg)

## Built around the rename itself

- **Keep the right numbers.** Recognized seasons, episode numbers, and gaps stay intact. Sequential numbering for unrecognized files is optional.
- **See the result first.** Compare original and proposed names, select a range, exclude files, and resolve name conflicts before applying anything.
- **Recover a batch.** Undo restores previous names, with checks for files changed since renaming and recovery for interrupted batches.
- **Start where your files are.** Right-click supported videos in File Explorer and choose **Rename with Anemorph**. Episodes and movies can share a batch.

Everything runs locally. No account, uploads, or video conversion.

## What's improved in 0.0.3

**Explorer integration and uninstall are the focus of this release.** Duplicate menu entries are removed. The menu handler runs in Anemorph's own process, and uninstall closes that process instead of forcing Explorer to restart. The integration check no longer gets stuck on Working after reinstall.

**Repair is built in.** Settings > Integration > Repair Anemorph reloads saved preferences, refreshes the current file list, and repairs menu registration. Repair and Uninstall now share one panel.

**Tested beyond the dedicated GPU.** With the process limited to two logical processors, a 50,000-filename preview took about **194 ms**. The UI also completed its checks on Microsoft's CPU renderer. These are measurements from a constrained modern PC, not a promise for every older machine. Read the [performance and security report](docs/performance.md) for memory figures, test conditions, and remaining limitations.

The release also adds **11 interface languages** and improves selection, button clarity, context menus, and small-window layouts. See the [full changelog](CHANGELOG.md#003---2026-10-28).

## Start renaming

1. Download **Anemorph-Setup-0.0.3.exe** from the [release page](https://github.com/yaboirar/Anemorph/releases/tag/v0.0.3). A portable ZIP is also available.
2. Choose a folder, paste its path, or drop it into the app.
3. Check the series name and preview, then include the files you want.
4. Click **Rename**. Use **Undo last batch** to restore the previous names.

| Original | Proposed |
| :--- | :--- |
| `[Group] Anemorph - 09 [1080p].mkv` | `Anemorph S01E09.mkv` |
| `[Group] Anemorph Movie [1080p].mp4` | `Anemorph Movie.mp4` |

The installer lets you choose its destination and includes Explorer integration. No .NET runtime is needed. The portable app works without installation; Explorer integration requires setup.

## In File Explorer

![A single Rename with Anemorph command in the Windows 11 context menu](assets/screenshots/explorer-menu.png)

Choose the classic menu or Windows 11's first right-click menu under **Settings > Integration**. The first-menu option may require Windows administrator approval to trust the included development signing certificate. Declining keeps the classic menu available. The installer is unsigned.

## Your library, your choices

Choose your episode/movie naming styles, classify a file from its right-click menu, and filter any combination of supported containers. No active filter shows all supported videos. Configuration controls which choices appear in Home's filter menu.

Supported containers: **MKV, MP4, MOV, AVI, WebM, M4V**, WMV, FLV, MPG, MPEG, MPE, M2V, TS, MTS, M2TS, VOB, OGV, OGM, 3GP, 3G2, ASF, DIVX, F4V, MXF, RM, and RMVB.

Languages: English (US/UK), Russian, Spanish, Portuguese, Turkish, German, Japanese, Korean, Simplified Chinese, and Modern Standard Arabic.

- [User guide](docs/usage.md): naming, selection, filters, undo, and installation.
- [Building from source](docs/building.md): Rust workspace, installer, and Windows integration.
- [Performance and security](docs/performance.md): measured results and their limits.
- [Contributing](CONTRIBUTING.md): bug reports, ideas, and code changes.

Created by [@yaboirar](https://github.com/yaboirar). Written in Rust.

Source code is licensed under [MIT](LICENSE). The supplied Anemo artwork is excluded from MIT; see [third-party notices](THIRD_PARTY_NOTICES.md) and [dependency licenses](DEPENDENCY_LICENSES.md).
