# Anemorph - Anime and TV Episode Batch Renamer

**Version 0.0.1** · Windows · Rust

Anemorph is a free, open-source bulk file renamer for anime and TV episodes on Windows. Batch rename MKV, MP4, AVI, M4V, and WebM files with automatic season and episode detection, custom filename formats, and a preview before applying changes.

```text
[Group] My Show - 09 [1080p].mkv
                  ↓
My Show S01E09.mkv
```

## What you can do

- Detect season and episode numbers from filenames and season folders.
- Create your own episode naming format, or use manual numbering.
- Filter MKV, MP4, AVI, M4V, and WebM files, including subfolders.
- Choose individual files or whole ranges and compare original and proposed names.
- Use dark/light themes, your own highlight color, and interface zoom.
- Undo a batch, with collision checks and recovery support for interrupted operations.

Anemorph changes filenames only. It does not convert videos, modify their contents, or fetch episode titles online. The app works locally without an account.

## Getting started

1. Open `Anemorph.exe` from a Windows release build. No installer or .NET runtime is required.
2. Paste a folder path, click **Browse**, or drop a folder into the window.
3. Check the suggested series name. You can edit it or use its refresh icon to restore a suggestion.
4. Review the **New filename** column and choose the files you want to rename.
5. Click **Rename**. **Undo last batch** restores the most recent batch when the files are still unchanged.

The small version label sits beneath Anemorph in the upper-left corner. Click the app name to open About.

**Home** is for browsing and renaming. **Settings** contains appearance, highlight color, and zoom. **Configuration** contains naming rules, detection, and folder options. The filter icon on Home offers the same container filters without leaving your files.

For selection, use Shift-click for a range, Ctrl-click for individual rows, Ctrl+A to highlight all rows, or Space to include/exclude highlighted rows. The header checkbox controls all ready files; **Unselect all** clears the selection.

## Your own filename format

Open **Configuration → Episode format**, edit the format, check the example, then click **Apply**. Anemorph adds the series name and original extension for you.

| Token | Meaning |
| --- | --- |
| `[S]`, `[Season]` | Literal `S` or `Season` |
| `[E]`, `[Episode]` | Literal `E` or `Episode` |
| `[SeaNum]` | Season number |
| `[EpsNum]` | Episode number |
| `[Num]` | Use at least two digits for following number tokens: `01`, `09` |
| `[FixNum]` | Use numbers without leading zeros: `1`, `9` |
| `[Space]` | Insert one space |

Choose `[Num]` or `[FixNum]` before inserting numbers. That choice stays active until you change it. Plain text and punctuation are kept.

| Format | Example output |
| --- | --- |
| `[Num][S][SeaNum][E][EpsNum]` | `My Show S01E09.mkv` |
| `[FixNum][S][SeaNum][Space][E][EpsNum]` | `My Show S1 E9.mkv` |
| `[Num][Episode][Space][EpsNum]` | `My Show Episode 09.mkv` |

## A few useful details

Preview first, especially when filenames are unusual. If a season cannot be detected, Anemorph uses the fallback season in Configuration. Ambiguous files and filename conflicts are flagged for review.

Undo history survives closing the app. New batches record file identity, size, and modification time, so Undo stops if a file was replaced or changed. An interrupted batch can be recovered through Undo before starting another batch. Keep your own backups for important files; recovery cannot guarantee protection from storage failure or outside changes during a batch.

Settings and undo history are stored in `%LOCALAPPDATA%\EpisodeRenamer`. That folder name is retained for compatibility with earlier builds. The files contain local paths, so avoid uploading them in bug reports. Remembering the last folder is optional.

The interface uses Arial Bold from Windows for the app name, with a fallback if unavailable. No proprietary font file is bundled. Rounded window corners depend on Windows support.

## Build from source

You need Windows, a stable Rust toolchain supporting edition 2024, and its Windows linker/build tools. This project has been tested with Rust 1.99.1 and the `x86_64-pc-windows-gnu` toolchain. An MSVC toolchain requires the corresponding Visual Studio C++ build tools for linking.

From the project folder:

```powershell
cargo build --release --locked
```

Your executable will be at `target/release/anemorph.exe`.

```powershell
cargo fmt --check
cargo clippy --locked -- -D warnings
cargo test --locked
```

For an isolated UI smoke test or performance benchmark, use a **dedicated scratch directory**. These commands generate their own fixture files and reports:

```powershell
target\release\anemorph.exe --smoke-test C:\Temp\Anemorph-smoke
target\release\anemorph.exe --benchmark C:\Temp\Anemorph-benchmark
```

The table renders visible rows only. File scanning, renaming, and large previews run on worker threads; repeated preview edits are coalesced. The interface sleeps when idle and keeps animations short.

## Contributing

Bug reports, ideas, and focused pull requests are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for what to include and how to check your changes. Please use invented example filenames instead of uploading videos or personal paths.

Release notes live in [CHANGELOG.md](CHANGELOG.md). The version in `Cargo.toml` also supplies the GUI label, About version, and Windows executable metadata.

## License and artwork

Anemorph's source code and documentation are available under the [MIT license](LICENSE). In plain language, you can use, modify, and share the code, including commercially, while keeping its license notice.

The supplied Anemo logo is **excluded from MIT**. No redistribution permission or open-source license has been established for that artwork; the code license does not grant rights to use or redistribute it. The original logo is retained at the maintainer’s request. Third-party dependency licenses also remain in effect. See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) and [DEPENDENCY_LICENSES.md](DEPENDENCY_LICENSES.md).
