<div align="center">

# User guide

**From a folder of videos to names that make sense.**

[Home](../README.md) · [Download](https://github.com/yaboirar/Anemorph/releases/latest) · [Report a problem](https://github.com/yaboirar/Anemorph/issues)

</div>

---

## Your first rename

1. Install **Anemorph-Setup-0.0.3.exe**, or extract the portable ZIP and open **Anemorph.exe**.
2. Browse for a folder, paste its path, or drag it into the window.
3. Check the suggested series name. Edit it, or use its refresh icon to restore the name detected from the folder.
4. Review the **New filename** column and check the files you want to include.
5. Click **Rename**. **Undo last batch** restores the previous names while the files are still unchanged.

Only filenames change. The app does not convert or re-encode your videos.

| Page | What it's for |
| :--- | :--- |
| **Home** | Open files, review new names, select files, and rename |
| **Configuration** | Set filename formats, detection rules, folder options, and visible filter choices |
| **Settings** | Change language and appearance, manage Explorer integration, repair, or uninstall |

Click **Anemorph** in the bottom-left corner to open About. The version is shown beneath the app name.

---

## Open videos from File Explorer

Select one or several supported videos, right-click, and choose **Rename with Anemorph**. If Anemorph is already running, the selection goes to that window.

**Those files arrive checked, including files detection cannot identify.** Being checked does not make an invalid filename ready to rename. Mark the file as an episode or movie to supply missing information, or leave it out.

Only the selected files are loaded. Refreshing, filtering, renaming, and undo keep that scope. Browse or enter a folder to return to folder mode.

### Choose the right-click menu

On Windows 11, open **Settings > Integration** to enable **First right-click menu**. Windows may ask for administrator approval to trust the included Anemorph development certificate in **Trusted People**. Cancelling keeps the option under **Show more options**.

The portable ZIP does not install Explorer integration. The installer adds it for supported video types without changing your default video player.

---

## Select and filter

| Action | Control |
| :--- | :--- |
| Include or exclude a file | Its checkbox |
| Include or exclude highlighted rows | **Include**, **Exclude**, or Space |
| Highlight a range | Shift-click |
| Highlight individual rows | Ctrl-click |
| Highlight every row | Ctrl+A |
| Check or clear files together | The header checkbox or **Select all / Unselect all** |

Checkmarks and highlighted rows are separate: highlighting chooses which rows an action applies to; checkmarks choose files to include in a rename. Files without a valid new name are not renamed.

Drag a column divider to resize it. Double-click the divider to reset its width.

### Filter by video type

On Home, open the filter icon and choose any combination of types. **No active filter means all supported video types.** Select all / Unselect all controls the visible choices; Shift-click selects or clears a range.

**Configuration > Home filter options** controls which choices appear in that menu. MKV, MP4, MOV, AVI, WebM, and M4V are shown by default. Hiding a choice also clears its active filter. It does not remove support for that video type.

---

## Your filename formats

Open **Configuration > Filename format**. Episodes and movies have separate inputs in the same panel. Edit a format, check its example, and click **Apply**.

### Episodes

Anemorph adds the series name and original extension. These indexes control the part between them:

| Format index | Meaning |
| :--- | :--- |
| `[S]`, `[Season]` | The text `S` or `Season` |
| `[E]`, `[Episode]` | The text `E` or `Episode` |
| `[SeaNum]` | Season number |
| `[EpsNum]` | Episode number |
| `[Num]` | Use at least two digits for following numbers: `01`, `09` |
| `[FixNum]` | Use numbers without leading zeros: `1`, `9` |
| `[Space]` | One space |

Choose `[Num]` or `[FixNum]` before inserting numbers. The choice applies to following numbers until changed. Plain text and punctuation are kept.

| Format | Example |
| :--- | :--- |
| `[Num][S][SeaNum][E][EpsNum]` | `Anemorph S01E09.mkv` |
| `[FixNum][S][SeaNum][Space][E][EpsNum]` | `Anemorph S1 E9.mkv` |
| `[Num][Episode][Space][EpsNum]` | `Anemorph Episode 09.mkv` |

### Movies

A standalone `Movie` marker in a filename identifies a movie, regardless of letter case. You can also right-click a file and choose **Mark as movie**.

| Format index | Meaning |
| :--- | :--- |
| `[Movie Name]` | Cleaned title taken from the filename |
| `[Movie]` | The text `Movie` |
| `[Space]` | One space |

The default `[Movie Name] Movie` turns `[Group] Anemorph Movie [1080p].mp4` into `Anemorph Movie.mp4`. Movie names do not use the series-name field or consume episode numbers.

Review titles containing the word Movie, since that word is also a detection marker. Keeping a Movie marker in the result helps automatic detection when you load it again. Episode-only indexes cannot be used in movie formats, and vice versa.

---

## When detection needs help

Right-click a file or highlighted selection:

| Choice | What it does |
| :--- | :--- |
| **Mark as movie** | Uses the movie filename format |
| **Mark as episode** | Keeps recognized numbers and asks for missing ones |
| **Use automatic detection** | Removes your manual classification |

For multiple episodes, known numbers stay unchanged; only unnumbered files receive the sequence you enter. Marking changes the preview first. Marks are saved locally and follow a successful rename or undo.

Recognized episode numbers and gaps are preserved even with automatic detection off. **Number unrecognized files sequentially** is a separate, optional setting. The fallback season is used when no season is found.

### Understand the status

| Status | Meaning |
| :--- | :--- |
| **Ready** | A valid new filename is available |
| **Title changed** | The series title changes while the episode suffix stays the same |
| **No change** | The filename already matches; no rename is needed |
| **Done** | Renamed successfully in this session |
| **Skipped** | Information is missing or a valid name could not be generated |
| **Name conflict** | The proposed filename would conflict with another file |

---

## Undo and recovery

Undo history survives closing the app. Undo checks file identity, size, and modification time, and stops if a file was changed or replaced. An interrupted batch can be recovered through Undo before starting another one.

Keep backups of important files. Undo cannot protect against storage failure or unrelated changes made during a batch.

**Settings > Integration > Repair Anemorph** reloads saved preferences, clears temporary interface state, checks the current folder again, and repairs Explorer registration. It preserves videos, history, and file marks. Unapplied color and format edits return to saved values. Missing or damaged app files require the installer's **Repair / reinstall** option.

---

## Installation and removal

| Task | What to expect |
| :--- | :--- |
| Install | Choose the destination folder; Start menu and Installed apps entries are added |
| Reinstall | Choose **Repair / reinstall** or **Uninstall**; settings stay unless you choose a reset |
| Uninstall | Removes Anemorph files, settings, history, marks, pending selections, shortcuts, and registration |
| Locked files | Reports the problem rather than forcing File Explorer to close |

You can open Uninstall from Settings or Windows Installed apps. Your videos and unrelated files are kept. Cleanup includes recorded previous install folders and exact Anemorph cache entries, but cannot promise removal of all Windows-managed records.

The installer is unsigned, so Windows may show an unknown-publisher or SmartScreen warning. Installing alone does not add certificate trust. The first-menu feature separately asks for approval if trust is needed; uninstall removes trust added by that feature and leaves pre-existing trust alone.

<details>
<summary><strong>Local settings and language details</strong></summary>

Settings, file marks, and undo history are stored in `%LOCALAPPDATA%\EpisodeRenamer`. The folder name is retained for compatibility. These files contain local paths, so do not upload them in bug reports.

Choose **Settings > Language** for English (US/UK), Russian, Spanish, Portuguese, Turkish, German, Japanese, Korean, Simplified Chinese, or Modern Standard Arabic. The choice is saved immediately. Filenames and format indexes remain literal; Windows dialogs use Windows' language.

Appearance includes highlight color and zoom. Turning animations off keeps the top tabs as icons. Arial Bold and other language fonts are loaded from Windows when available; no Windows or LEMON MILK font is bundled. Rounded corners depend on Windows support.

</details>

---

[Back to Anemorph](../README.md) · [Report a problem](https://github.com/yaboirar/Anemorph/issues)
