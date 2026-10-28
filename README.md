<div align="center">

# Anemorph

**Your videos. Your naming style.**

Rename anime episodes, TV shows, and movies in bulk. Preview every change first, and undo the last batch if you change your mind.

[![Release](https://img.shields.io/github/v/release/yaboirar/Anemorph?style=for-the-badge&label=Release)](https://github.com/yaboirar/Anemorph/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/yaboirar/Anemorph/total?style=for-the-badge&label=Downloads)](https://github.com/yaboirar/Anemorph/releases)
[![Platform](https://img.shields.io/badge/Platform-Windows-0078D6?style=for-the-badge)](https://github.com/yaboirar/Anemorph/releases/latest)
[![License](https://img.shields.io/badge/License-MIT-10b981?style=for-the-badge)](LICENSE)

[**Download the installer**](https://github.com/yaboirar/Anemorph/releases/download/v0.0.3/Anemorph-Setup-0.0.3.exe) · [Portable ZIP](https://github.com/yaboirar/Anemorph/releases/download/v0.0.3/Anemorph-v0.0.3-windows.zip) · [User guide](docs/usage.md) · [Changelog](docs/changelog.md)

<br />

![Anemorph showing original filenames beside their new names](assets/screenshots/home.jpg)

</div>

---

## The problem

Downloaded episodes arrive with release-group tags, resolution labels, and inconsistent numbering. Renaming a season by hand is slow, and one typo can break your library order. Generic batch renamers make you write patterns, and they rarely understand the difference between an episode and a movie.

Anemorph is built for this one job.

| Before | After |
| :--- | :--- |
| `[Group] Anemorph - 09 [1080p].mkv` | `Anemorph S01E09.mkv` |
| `[Group] Anemorph - 19 [1080p].mkv` | `Anemorph S01E19.mkv` |
| `[Group] Anemorph Movie [1080p].mp4` | `Anemorph Movie.mp4` |

Anemorph changes filenames only. It never converts, re-encodes, or uploads your videos, and no account is needed.

---

## How it works

Every rename follows the same four steps. Your video filenames stay unchanged until the last one.

1. **Open a folder.** Browse for it, paste its path, or drag it into the window.
2. **Detect.** Anemorph reads the filenames to identify episodes, movie markers, and episode numbers.
3. **Review.** Original and new names sit side by side. Include or exclude any file, fix anything detection missed, and resolve conflicting names before you continue.
4. **Rename.** One click applies the batch. If it isn't what you wanted, **Undo last batch** restores the previous names.

---

## Your naming style

Episodes and movies each get their own format, so a season and a standalone film can follow different rules in the same library.

| Setting | Example result |
| :--- | :--- |
| Leading zeros on | `Anemorph S01E09.mkv` |
| Leading zeros off | `Anemorph S1E9.mkv` |
| Movie format | `Anemorph Movie.mp4` |

You control spacing and zero-padding for episode numbers, and the choice is saved for next time. Open **Configuration** to set it up.

---

## Features

**Preview before anything changes**
Compare original and new names in one table. Include or exclude files individually, and see conflicting names flagged before you commit.

**Episode numbers stay true**
Anemorph keeps recognized numbers. If you own episodes 1, 4, and 19, they stay 1, 4, and 19. Numbering unrecognized files in sequence is a separate option you choose to enable.

**Manual override when detection misses**
Right-click any file to mark it as a movie or an episode and supply a missing episode number.

**Undo the last batch**
Restores the previous filenames as long as the renamed files are still unchanged.

**Filter by video type**
Show several formats at once on the Home screen, or leave the filter empty to see every supported type. Choose which types appear in the filter menu under **Configuration**.

**Built into File Explorer**
Select videos, right-click, and choose **Rename with Anemorph**.

**Multilingual interface**
English (US/UK), Russian, Spanish, Portuguese, Turkish, German, Japanese, Korean, Simplified Chinese, and Modern Standard Arabic.

**Wide format support**
26 video types. See the [full list](#supported-video-types).

---

## Install

| | Installer | Portable ZIP |
| :--- | :--- | :--- |
| Download | [Anemorph-Setup-0.0.3.exe](https://github.com/yaboirar/Anemorph/releases/download/v0.0.3/Anemorph-Setup-0.0.3.exe) | [Anemorph-v0.0.3-windows.zip](https://github.com/yaboirar/Anemorph/releases/download/v0.0.3/Anemorph-v0.0.3-windows.zip) |
| Install location | The folder you choose | None, run from anywhere |
| File Explorer right-click | Yes | No |
| Setup | Run the installer | Extract and open **Anemorph.exe** |

> **Windows warning.** The current installer is unsigned, so Windows may show an unknown-publisher or SmartScreen warning. Only download from this repository's [Releases page](https://github.com/yaboirar/Anemorph/releases). The source is public if you'd rather [build it yourself](docs/building.md).

---

## Start from File Explorer

Select your videos, right-click, and choose **Rename with Anemorph**. Anemorph opens with those files ready to preview.

**Updated in the latest 0.0.3 patch:** selections open in your existing Anemorph window. Files stay checked even when detection cannot identify an episode or movie, including after refresh. Files without a valid new name remain skipped until you resolve them.

![Rename with Anemorph in the Windows 11 right-click menu](assets/screenshots/explorer-menu.png)

On Windows 11, **Settings > Integration** lets you choose between the first right-click menu and **Show more options**. Enabling the first-menu option may require Windows approval.

---

## Configuration

| Where | What you can change |
| :--- | :--- |
| **Configuration** | Episode and movie filename formats, and which video types appear in the Home filter menu |
| **Settings** | Interface language, appearance, and Explorer integration |
| **Settings > Repair Anemorph** | Fixes saved settings or Explorer integration that stopped working |

The [user guide](docs/usage.md) walks through each option.

### Supported video types

MKV, MP4, MOV, AVI, WebM, M4V, WMV, FLV, MPG, MPEG, MPE, M2V, TS, MTS, M2TS, VOB, OGV, OGM, 3GP, 3G2, ASF, DIVX, F4V, MXF, RM, and RMVB.

---

## FAQ

<details>
<summary><strong>Does it modify my video files?</strong></summary>

No. Only the filename changes. Nothing is converted, re-encoded, or uploaded.

</details>

<details>
<summary><strong>Does it need an account or internet connection?</strong></summary>

It needs no account. Renaming happens locally on your files.

</details>

<details>
<summary><strong>What if it detects a file wrongly?</strong></summary>

Right-click the file, mark it as a movie or an episode, and enter the episode number yourself. You can also leave any file out of the batch.

</details>

<details>
<summary><strong>What happens to missing episodes?</strong></summary>

They stay missing. Anemorph keeps the numbers your files already have and never fills gaps or shifts later episodes down.

</details>

<details>
<summary><strong>When can I undo a rename?</strong></summary>

**Undo last batch** restores the previous names while the renamed files are still unchanged. If you have since renamed or moved them yourself, the undo may no longer apply.

</details>

<details>
<summary><strong>Does it work on macOS or Linux?</strong></summary>

Anemorph currently supports Windows only. Building the source does not add macOS or Linux support.

</details>

<details>
<summary><strong>How fast is it on large folders?</strong></summary>

See [test results and limits](docs/performance.md) for measured speed and memory use, including large-folder and rename/undo tests.

</details>

---

## Troubleshooting

| Problem | What to try |
| :--- | :--- |
| Windows blocks or warns about the installer | The installer is unsigned. Choose more info, then run anyway, if the file came from the official Releases page. |
| Right-click option is missing | Use the installer (the portable ZIP doesn't add it). On Windows 11, check **Settings > Integration** and look under **Show more options**. |
| Settings or Explorer integration stopped working | Open **Settings > Repair Anemorph**. |
| A file was detected as the wrong type or number | Right-click it and set the type and episode number manually. |
| Two files would get the same name | Resolve the flagged conflict in the preview before renaming. |
| Anything else | [Open an issue](https://github.com/yaboirar/Anemorph/issues) with what you did and what you expected. |

---

## Project

[User guide](docs/usage.md) · [Changelog](docs/changelog.md) · [Build from source](docs/building.md) · [Contribute](docs/contributing.md) · [Test results and limits](docs/performance.md) · [Report a problem or suggest an idea](https://github.com/yaboirar/Anemorph/issues)

Created by [@yaboirar](https://github.com/yaboirar).

### License

The source code is licensed under [MIT](LICENSE). The supplied Anemo artwork is excluded from that license; redistribution permission has not been established. See [third-party notices](docs/licenses/third-party-notices.md) and [dependency licenses](docs/licenses/dependency-licenses.md).
