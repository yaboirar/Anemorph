# Anemorph

A free, open-source batch renamer for anime, TV episodes, and movies on Windows. Turn release filenames into a consistent library, with a preview before anything changes.

[Download v0.0.2](https://github.com/yaboirar/Anemorph/releases/tag/v0.0.2) · [User guide](docs/usage.md) · [Changelog](CHANGELOG.md)

| Before | After |
| :--- | :--- |
| `[Group] Anemorph - 09 [1080p].mkv` | `Anemorph S01E09.mkv` |
| `[Group] Anemorph Movie [1080p].mp4` | `Anemorph Movie.mp4` |

## Built for your library

- **Keep the right episode numbers.** Detect seasons and episodes without closing gaps or renumbering recognized files.
- **Define your own filenames.** Customize episode and movie formats together, with readable tokens and live examples.
- **Handle episodes and movies in one batch.** Detect movie markers automatically, or classify files from the right-click menu.
- **Review before renaming.** Compare names side by side, resize columns, select ranges, and see conflicts or unchanged files.
- **Start from File Explorer.** Send selected videos straight to Anemorph through **Rename with Anemorph**.
- **Undo the last batch.** Restore previous names, with checks for files that changed after renaming.

Anemorph works locally without an account. It changes filenames only; it does not convert or modify video contents.

## Get started

1. Download **Anemorph-Setup-0.0.2.exe** from the [release page](https://github.com/yaboirar/Anemorph/releases/tag/v0.0.2), or choose the portable ZIP.
2. Choose a folder, paste its path, or drop it into Anemorph.
3. Check the series name and proposed filenames, then select your files.
4. Click **Rename**.

The installer lets you choose a destination and includes repair, uninstall, and Explorer integration. No .NET runtime is needed. The portable build works without installation; Explorer integration requires setup.

On Windows 11, the Explorer command is available under **Show more options**. To enable it on the first menu, open **Settings > Integration**. Windows may request administrator approval to trust the included development signing certificate. Cancelling keeps the classic menu available. The installer itself is unsigned.

## Choose your format

Open **Configuration > Filename format**. Episode and movie formats share one panel and an **Apply** action.

| Format | Preview |
| :--- | :--- |
| `[Num][S][SeaNum][E][EpsNum]` | `Anemorph S01E09.mkv` |
| `[FixNum][S][SeaNum][Space][E][EpsNum]` | `Anemorph S1 E9.mkv` |
| `[Movie Name] Movie` | `Anemorph Movie.mkv` |

For episodes, Anemorph adds the series name and preserves the original extension. Movies use their cleaned filename title. See the [token reference and naming guide](docs/usage.md#your-own-filename-format) for the full options.

## Supported videos

MKV, MP4, MOV, AVI, WebM, M4V, WMV, FLV, MPG, MPEG, MPE, M2V, TS, MTS, M2TS, VOB, OGV, OGM, 3GP, 3G2, ASF, DIVX, F4V, MXF, RM, and RMVB.

Select any combination in Home's filter menu. No active filter shows every supported container. Configuration controls which choices appear in that menu.

## More

- [Using Anemorph](docs/usage.md): selection, media markers, filters, undo, and installation.
- [Building from source](docs/building.md): Rust workspace, installer, and Windows integration.
- [Contributing](CONTRIBUTING.md): bug reports, ideas, and code changes.

Created by [@yaboirar](https://github.com/yaboirar). Written in Rust.

Source code is licensed under [MIT](LICENSE). The supplied Anemo artwork is excluded from MIT; see [third-party notices](THIRD_PARTY_NOTICES.md) and [dependency licenses](DEPENDENCY_LICENSES.md).
