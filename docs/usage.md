# Using Anemorph

## Getting started

1. Run `Anemorph-Setup-0.0.2.exe` to install, or open the portable `Anemorph.exe`. Neither needs a .NET runtime.
2. Paste a folder path, click **Browse**, or drop a folder into the window.
3. Check the suggested series name. You can edit it or use its refresh icon to restore the original detected name, including after you edit or clear it.
4. Review the **New filename** column and choose the files you want to rename.
5. Click **Rename**. **Undo last batch** restores the most recent batch when the files are still unchanged.

The small version label sits beneath Anemorph in the upper-left corner. Click the app name to open About, including the creator credit to [@yaboirar](https://github.com/yaboirar).

**Home** is for browsing and renaming. **Settings** contains appearance, highlight color, zoom, and the uninstall button at the bottom. **Configuration** contains naming rules, detection, season/manual numbering, and folder options. Home's filter menu narrows the file list without leaving your files. Nothing highlighted means all supported video types are shown. Choose any combination of containers to show their matching files. Each choice toggles independently. Select all / Unselect all controls all visible choices; Shift-click selects or clears a range.

**Configuration → Home filter options** controls which choices appear in Home's filter menu. MKV, MP4, MOV, AVI, WebM, and M4V are enabled by default. These configuration switches control menu visibility; Home's selections control the active filter. Turning off a choice removes it from the menu and clears that choice if it was active. With no active Home filter, all supported containers are shown, including types that are not listed in the menu.

Right-click a file to **Mark as movie**, **Mark as episode**, or return to **Use automatic detection**. Mark as episode preserves recognized numbers. If a number is missing, a small Season/Episode dialog lets you supply it; for a selection, known numbers stay unchanged and only unnumbered files receive the new sequence. Marking changes the preview, not the filename. Manual marks are saved locally and follow a successful rename or Undo. Right-clicking a highlighted selection applies the mark to that selection; otherwise it applies to the clicked file.

Drag the dividers in the table header to resize the filename and status columns. Widths are remembered, and double-clicking a divider resets it. The layout adapts to window size.

For selection, use Shift-click for a range, Ctrl-click for individual rows, Ctrl+A to highlight all rows, or Space to include/exclude highlighted rows. The header checkbox controls all ready files; **Unselect all** clears the selection.

## Your own filename format

Open **Configuration → Filename format → Episodes**, edit the formats, check the examples, then click **Apply**. Episode and movie formats share one panel and one Apply/Reset row. Anemorph adds the series name and original extension for you.

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

### Movie filenames

Files whose names contain the standalone word `Movie` (case-insensitive) are categorized as movies. Movie files use **Configuration → Filename format → Movies**, independently of episode numbering and the series-name field. They do not consume a number in manual episode mode.

The default is `[Movie Name] Movie`: `[Group] Anemorph Movie [1080p].MOV` becomes `Anemorph Movie.MOV`. The title is taken from the filename, with release-group tags, common technical tags, and the movie marker removed. Review the preview when a real title contains the word Movie. Only the original file extension is kept; video data is not inspected or converted.

| Movie token | Meaning |
| --- | --- |
| `[Movie Name]` | Cleaned movie title from the filename |
| `[Movie]` | Literal `Movie` |
| `[Space]` | One space |

Plain text is supported, and movie format changes require **Apply**. Keep the word Movie in the result if you want automatic movie classification when loading the renamed files again. A movie title that cannot be determined is skipped for review.

## A few useful details

Recognized season and episode numbers are preserved, including gaps, even when automatic detection is off. Turning detection off does not renumber your files. Unrecognized files are skipped unless you explicitly enable **Number unrecognized files sequentially** in Configuration; this opt-in fills only files without recognized numbering.

Unchanged filenames show **No change** and are excluded from Rename. Files successfully renamed in the current session show **Done**, including after refreshing the file list. Undo clears that completed state. **Title changed** means the episode suffix stays the same but the series-name text changes. Season/episode pairs such as `S3 - 23` are recognized as `S03E23`.

Preview first, especially when filenames are unusual. If a season cannot be detected, Anemorph uses the fallback season in Configuration. Ambiguous files and filename conflicts are flagged for review.

Undo history survives closing the app. New batches record file identity, size, and modification time, so Undo stops if a file was replaced or changed. An interrupted batch can be recovered through Undo before starting another batch. Keep your own backups for important files; recovery cannot guarantee protection from storage failure or outside changes during a batch.

Settings, manual media marks, and undo history are stored in `%LOCALAPPDATA%\EpisodeRenamer`. That folder name is retained for compatibility with earlier builds. The files contain local paths, so avoid uploading them in bug reports. Remembering the last folder is optional.

The interface uses Arial Bold from Windows for the app name, with a fallback if unavailable. No proprietary font file is bundled. Rounded window corners depend on Windows support.

## Windows installation and Explorer menu

The installer adds Anemorph to **Installed apps** and the Start menu for your Windows account. Choose the installation drive/folder on **Choose Install Location**, using its path field or **Browse**. A desktop shortcut is optional.

If Anemorph is already installed, setup first offers **Repair / reinstall** or **Uninstall**. Repair replaces the app files and restores its shortcuts and integration; it can also move the installation to a different folder. Settings stay unless you select **Reset settings for a fresh start**. Repair works even if app files or the old uninstaller are missing or damaged.

Uninstall removes the app files, shortcuts, Explorer registration, and all Anemorph data in `%LOCALAPPDATA%\EpisodeRenamer`, including settings, undo history, media markers, and pending selections. Your videos and unrelated files are kept. If Windows locks an app file, cleanup reports the problem and allows retry rather than claiming a complete uninstall. Close Anemorph before repair or uninstall.

You can also open the same uninstaller from the bottom of **Settings**. Anemorph closes before the uninstaller begins; Windows shows its uninstall confirmation before removing anything. Cleanup scans recorded previous installation folders and exact Anemorph executable entries in the per-user app-name and compatibility caches. Unrelated files and other apps' cache entries are kept. Windows-managed shared logs are outside this cleanup's scope; a zero-trace guarantee is not made.

File Explorer integration is built into Anemorph and installed automatically. It adds **Rename with Anemorph** for the supported video extensions. On Windows 11, right-click a video and choose **Show more options** to find it. Select multiple videos to open them together in one preview. Other files are excluded from the command. Your default video player stays unchanged.

To put Anemorph on Windows 11's first right-click menu, open **Settings > Integration** and click **Enable** beside **First right-click menu**. Anemorph handles registration. If the included development certificate needs trust, Windows asks for administrator approval; this adds only the embedded Anemorph signing certificate to the computer's Trusted People store. Cancelling keeps the classic menu available. **Disable** switches back. Uninstall removes a certificate added by this feature and may request administrator approval for that cleanup; it leaves pre-existing certificate trust alone.

The first Windows 11 menu uses the optional signed identity package described below. If the package is missing or Windows cannot trust or register it, installation still completes with the classic menu. Certificate trust is not required to use Anemorph. Repair and uninstall unregister Anemorph's identity before replacing or removing its files.

Opening a selection loads only those files. Refresh, container filtering, rename, and undo retain that scope. Browse or enter a folder to return to folder mode. The integration does not run a background service and never renames anything before you click **Rename**.

This local installer is unsigned. Windows may display an unknown-publisher or SmartScreen prompt. It does not install a signing certificate or change Windows security settings.

If Explorer holds Anemorph's menu DLL during uninstall, setup offers to restart Explorer and release it before removing app files or settings. The taskbar briefly disappears and open folder windows may close. Cancelling stops removal. Silent uninstall reports a lock instead of restarting Explorer without permission.
