<div align="center">

# Third-party notices

**Which licenses apply to the parts included with Anemorph.**

[Home](../../README.md) · [Code license](../../LICENSE) · [Dependency licenses](dependency-licenses.md)

</div>

---

| Part | Where to find its terms |
| :--- | :--- |
| Anemorph source code | [MIT license](../../LICENSE) |
| Supplied Anemo artwork | Separate notice below; redistribution permission is not established |
| Installer components | [NSIS license notices](https://github.com/yaboirar/Anemorph/blob/main/installer/NSIS-LICENSE.txt) |
| Rust dependencies and fallback fonts | [Dependency licenses](dependency-licenses.md) |

---

## Installer

The installer uses unmodified NSIS components, including Modern UI, the System plug-in, and the LZMA compression module. Their notices and license terms are in `installer/NSIS-LICENSE.txt` in the source and `NSIS-LICENSE.txt` in the installed app folder. NSIS source is available at https://github.com/kichik/nsis. Anemorph source remains under its own MIT license.

## Anemo logo

`assets/icon.svg`, `assets/icon.png`, and `assets/icon.ico` contain the Anemo emblem supplied for this project, associated with Genshin Impact. These files are not original Anemorph artwork and are excluded from the project's MIT license. No redistribution permission or open-source asset license has been established. The supplied logo is retained at the maintainer's request. The code license does not grant redistribution rights to this artwork. Its presence does not imply affiliation or endorsement.

## Fonts

Anemorph requests Arial Bold from the user's Windows installation at runtime. CJK and Arabic interfaces also load a suitable installed Windows font only when selected. No Windows or LEMON MILK font file is included in the project or executable. If Arial Bold is unavailable, the interface uses egui's bundled fallback fonts. Installed fonts remain subject to their own licenses.

## Rust dependencies

Dependencies and bundled fallback fonts retain their upstream licenses. See Cargo.lock for exact dependency versions and each upstream package's license files. The project's MIT license does not replace those licenses. The Windows normal/build dependency notices from this locked build are collected in [dependency licenses](dependency-licenses.md); review upstream terms when changing dependencies or distribution targets.

---

[Back to Anemorph](../../README.md) · [Contributing assets](../contributing.md#licenses-and-assets)
