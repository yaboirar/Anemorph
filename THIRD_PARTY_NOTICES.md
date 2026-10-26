# Third-party assets and dependencies

## Anemo logo — not covered by the MIT license

`assets/icon.svg`, `assets/icon.png`, and `assets/icon.ico` contain the Anemo emblem supplied for this project, associated with Genshin Impact. These files are not original Anemorph artwork and are excluded from the project's MIT license. No redistribution permission or open-source asset license has been established. The supplied logo is retained at the maintainer's request. The code license does not grant redistribution rights to this artwork. Its presence does not imply affiliation or endorsement.

## Fonts

Anemorph requests Arial Bold from the user's Windows installation at runtime. No Arial or LEMON MILK font file is included in the project or executable. If Arial Bold is unavailable, the interface uses egui's bundled fallback fonts. Installed fonts remain subject to their own licenses.

## Rust dependencies

Dependencies and bundled fallback fonts retain their upstream licenses. See Cargo.lock for exact dependency versions and each upstream package's license files. The project's MIT license does not replace those licenses. The Windows normal/build dependency notices from this locked build are collected in DEPENDENCY_LICENSES.md; review upstream terms when changing dependencies or distribution targets.
