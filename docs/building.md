<div align="center">

# Build from source

**Build Anemorph, run its checks, and package a Windows release.**

[Home](../README.md) · [Contributing](../CONTRIBUTING.md) · [Test results](performance.md) · [Signing](signing.md)

</div>

---

## What you need

| Tool | Purpose |
| :--- | :--- |
| Windows | The app and Explorer integration currently target Windows |
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

## Build the installer

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

[Contributing](../CONTRIBUTING.md) · [Signing a release](signing.md) · [Back to Anemorph](../README.md)
