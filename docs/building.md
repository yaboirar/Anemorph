<div align="center">

# Build from source

**Build Anemorph, run its checks, and package the app.**

[Home](../README.md) · [Contributing](contributing.md) · [Test results](performance.md) · [Signing](signing.md)

</div>

---

## What you need

| Tool | Purpose |
| :--- | :--- |
| Windows or Linux | Explorer integration and the Windows installer require Windows |
| Stable Rust with edition 2024 support | Builds the app and helpers |
| Windows linker and build tools | GNU tools or Visual Studio C++ build tools for your Rust target |
| NSIS | Packages the installer |
| Windows SDK tools | Needed when building the optional Windows 11 menu identity |

The project has been tested with Rust 1.99.1 and `x86_64-pc-windows-gnu`. Using an MSVC target requires its corresponding Visual Studio build tools.

## Build the app

From the repository folder:

```powershell
cargo build --release --locked --workspace
```

| Output in `target/release` | Purpose |
| :--- | :--- |
| `anemorph.exe` | Main app |
| `anemorph-broker.exe` | Separate Explorer menu host |
| `anemorph_shell.dll` | Shared shell implementation |
| `anemorph-maintenance.exe` | Installation and cleanup helper |

Normal launches share one window per Windows user/session. A later launch forwards its file selection to that window. The handoff uses a message-only Windows window and a named lock, so Windows package identity does not split it into separate app instances. It does not poll the filesystem while idle.

To open specific files:

```powershell
target\release\anemorph.exe --files "C:\Videos\Anemorph S01E01.mkv" "C:\Videos\Recording.mp4"
```

---

## Linux

The Linux build is a native desktop app. The default drawing path does not require a graphics card. X11 and Wayland are enabled; desktop testing currently covers Ubuntu 24.04 with Xfce on X11.

On Ubuntu 24.04, install stable Rust and these build/runtime libraries:

```sh
sudo apt install build-essential pkg-config libx11-dev libxkbcommon-dev libxkbcommon-x11-0 libwayland-dev libgl1-mesa-dev libegl1-mesa-dev libdbus-1-dev
cargo build --release --locked -p anemorph
cargo test --locked -p anemorph
cargo clippy --locked -p anemorph --all-targets -- -D warnings
```

Run `target/release/anemorph`, or open selected videos with `target/release/anemorph --files /path/to/video.mkv`. Later launches pass their files to the existing window. Settings and undo history are stored under `${XDG_DATA_HOME:-$HOME/.local/share}/anemorph`.

Create a portable archive with `sh installer/linux/package.sh`. Extract it, then run `./anemorph` or `sh install.sh` to add it to your app menu. You can choose another location with `sh install.sh /absolute/install/path`. A custom location may need its `share` folder added to `XDG_DATA_DIRS` for desktop menu discovery. Use `sh uninstall.sh` with the same location to remove installed app files; saved settings and undo history are kept.

The launcher also makes Anemorph available through compatible file managers' **Open With** choices. The Windows-only Explorer menu and uninstaller are not shown on Linux. Linux archives built on Ubuntu 24.04 require glibc 2.39 or newer, plus `libxkbcommon-x11-0` on X11. Wayland is compiled but has not been tested in a Wayland desktop session.

---

## Build the installer

The Linux packaging script also creates a `.deb` when `dpkg-deb` is available. Install the downloaded file with `sudo apt install ./Anemorph-0.0.4-linux-amd64.deb`; remove it with `sudo apt remove anemorph`. Package-managed app files are removed; your saved settings and undo history are kept. The package uses standard system folders. Use the portable installer if you prefer a custom location.

Install [NSIS](https://nsis.sourceforge.io/Download), put `makensis.exe` on PATH, and run:

```powershell
.\installer\build.ps1
```

The installer appears in `dist/`. Its version comes from `Cargo.toml`; registered video types come from `src/containers.rs`. Building does not install the app.

| Option | Use |
| :--- | :--- |
| `-TargetDir` | Choose the Cargo output directory |
| `-OutputDir` | Choose where the installer is written |
| `-SkipBuild` | Package binaries already built |
| `-ModernPackage` | Include the signed Windows 11 menu identity |
| `-ReleaseCertificateThumbprint` | Sign using a verified publisher certificate |
| `-SignTool` | Supply the Windows SDK signing tool path |

### Optional first right-click menu

This needs the Windows SDK and a code-signing certificate in the current user's **My** certificate store. Its subject must match `-Publisher` (default `CN=yaboirar`).

```powershell
.\installer\build-modern.ps1 -SdkBin "C:\path\to\Windows SDK\x64" -TargetDir target -CertificateThumbprint "YOUR_SIGNING_CERTIFICATE_THUMBPRINT"
.\installer\build.ps1 -SkipBuild -TargetDir target -ModernPackage target\modern-integration\Anemorph.identity.msix
```

The identity package references the externally installed app, so users can still choose an install folder. The installer checks the handler and version, registers the package, and removes it during uninstall.

Windows must trust the signing certificate. A self-signed certificate is for development and works only where its owner explicitly approves trust. Running `build-modern.ps1` without a thumbprint creates an unsigned inspection package that the installer refuses to bundle as signed.

<details>
<summary><strong>Development certificate setup</strong></summary>

For builds with the in-app trust action, export only the public certificate to a `.cer` file. Set `ANEMORPH_INTEGRATION_CERT` to its absolute path and rebuild the workspace before building the identity package.

The maintenance helper embeds those public bytes, limiting its trust action to that certificate. Never supply or commit a private key or PFX file. Builds signed by an already-trusted publisher do not need this development trust action.

</details>

For production signing, see the [signing guide](signing.md).

---

## Run the checks

```powershell
cargo fmt --all --check
cargo clippy --workspace --locked --all-targets -- -D warnings
cargo test --workspace --locked
```

The automated activation test launches a second test process and checks that its selection reaches the first process without acquiring another primary lock. It does not control the desktop.

### Performance checks

**Use a dedicated scratch directory.** These commands create disposable fixtures and reports. Never point them at a real media folder.

```powershell
target\release\anemorph.exe --benchmark C:\Temp\Anemorph-benchmark
target\release\anemorph.exe --smoke-test C:\Temp\Anemorph-smoke
```

| Command | Checks |
| :--- | :--- |
| `--benchmark` | Scanning, previews, rename/undo, and repeated-preview memory samples |
| `--smoke-test` | Interface paths, screenshots, and drawing measurements |
| `--renderer=wgpu` | Earlier DX12 renderer for diagnosis |
| `--renderer=glow` | Earlier OpenGL renderer for diagnosis |
| `--software-rendering` | Earlier DX12 WARP path, not the current default renderer |

Normal startup draws into a CPU pixel buffer without creating a GPU device. The table builds only visible rows, scans and large previews use background workers, and the interface sleeps when idle.

Smoke settings are isolated inside the output directory. Reports include `graphics.json` and `smoke.json`. Restricting a test process to two logical processors limits concurrency; it does not emulate an older CPU, slow disk, or limited RAM. See [measurements and limits](performance.md).

---

[Contributing](contributing.md) · [Signing a release](signing.md) · [Back to Anemorph](../README.md)
