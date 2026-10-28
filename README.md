<div align="center">

# Anemorph

**A fast, lightweight batch file renamer for anime and TV episode libraries on Windows.**

[![Language](https://img.shields.io/badge/Language-Rust-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![GUI](https://img.shields.io/badge/GUI-egui-blue?style=flat-square)](https://github.com/emilk/egui)
[![Platform](https://img.shields.io/badge/Platform-Windows-0078D6?style=flat-square&logo=windows)](https://github.com/yaboirar/Anemorph)
[![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)](LICENSE)

</div>

---

## Overview

Downloaded anime and TV show releases often come cluttered with fansub tags, video codec metadata, CRC32 hashes, and inconsistent episode numbering (e.g., `[SubGroup] Show_Name - 04v2 [1080p][HEVC][ABCD1234].mkv`). Media servers like Jellyfin, Plex, and Kodi frequently struggle to parse these raw filenames cleanly.

**Anemorph** is a native Windows desktop utility written in Rust using `egui` that strips the noise and standardizes your media library in seconds. It automatically detects season and episode numbers, previews your target naming format in real time, and lets you revert changes instantly if something doesn't look right.

## Key Features

- **Smart Season & Episode Detection:** Automatically parses episode numbers, multi-season formats (`S01E01`, `01x01`, `- 01`), and release versions while ignoring resolution tags (`1080p`, `720p`) and CRC hashes.
- **Custom Naming Formats:** Define your own output pattern using dynamic tokens so your files match your exact media server conventions.
- **Live Dry-Run Preview:** Inspect a side-by-side comparison of original filenames and their target names before touching a single file on disk.
- **One-Click Undo:** Made a typo in the series title? Revert the entire last batch operation immediately to restore original filenames.
- **Format Agnostic:** Built with `.mkv` and `.mp4` video containers in mind, alongside external subtitle files (`.ass`, `.srt`).
- **Native Performance:** Powered by Rust and `egui`—starts instantly with minimal memory footprint and no heavy webview/Electron overhead.

## Transformation Example

```text
Before:
├── [SubsPlease] Cyberpunk Edgerunners - 01 (1080p) [9F3A1B2C].mkv
├── [SubsPlease] Cyberpunk Edgerunners - 02 (1080p) [4D2E8F1A].mkv
└── [SubsPlease] Cyberpunk Edgerunners - 03v2 (1080p) [7C9B4E3D].mkv

After:
├── Cyberpunk Edgerunners - S01E01.mkv
├── Cyberpunk Edgerunners - S01E02.mkv
└── Cyberpunk Edgerunners - S01E03.mkv
