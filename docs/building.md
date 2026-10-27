# Building Anemorph

You need Windows, a stable Rust toolchain supporting edition 2024, and its Windows linker/build tools. This project has been tested with Rust 1.99.1 and the `x86_64-pc-windows-gnu` toolchain. An MSVC toolchain requires the corresponding Visual Studio C++ build tools for linking.

From the project folder:

```powershell
cargo build --release --locked --workspace
```

Your executable will be at `target/release/anemorph.exe`. The workspace also builds the Rust Explorer handler, `anemorph_shell.dll`, and the installer-only `anemorph-maintenance.exe` cleanup helper.

To build the installer, install [NSIS](https://nsis.sourceforge.io/Download), put `makensis.exe` on PATH, and run `installer/build.ps1` from PowerShell. The installer appears in `dist/`. The script takes its version from `Cargo.toml` and its registered extensions from `src/containers.rs`. You can supply `-TargetDir` and `-OutputDir` to use other build/output folders. Building the installer does not install the app.

For the first Windows 11 context menu, also install the Windows SDK build tools and supply a code-signing certificate in your current-user **My** certificate store. Its subject must match `-Publisher` (default `CN=yaboirar`):

For a development build with the in-app trust action, export only the public certificate to a `.cer` file, set `ANEMORPH_INTEGRATION_CERT` to its absolute path, and rebuild the workspace before building the identity package. The maintenance helper embeds those public certificate bytes, so elevation can trust only that fixed certificate. Never point this variable at a private key or PFX file. Builds signed by an already-trusted publisher do not need the development trust action.

```powershell
.\installer\build-modern.ps1 -SdkBin "C:\path\to\Windows SDK\x64" -TargetDir target -CertificateThumbprint "YOUR_SIGNING_CERTIFICATE_THUMBPRINT"
.\installer\build.ps1 -SkipBuild -TargetDir target -ModernPackage target\modern-integration\Anemorph.identity.msix
```

The identity package references the externally installed app and DLL, so installation-folder selection still works. The installer validates that its handler and version match the app, registers it for the current user on Windows 11, and removes it on uninstall. It does not add a trusted certificate or change Windows security policy. Windows must already trust the signing certificate. Production releases need a trusted signing certificate; a self-signed development certificate is useful only on machines whose owner explicitly trusts it. Never commit private signing keys or export them into release packages. Running `build-modern.ps1` without a thumbprint creates an unsigned package for inspection, which the installer refuses to bundle as a signed release.

To open an explicit selection from a terminal:

```powershell
anemorph.exe --files "C:\Videos\Show S01E01.mkv" "C:\Videos\Show S01E02.mp4"
```

```powershell
cargo fmt --all --check
cargo clippy --workspace --locked --all-targets -- -D warnings
cargo test --workspace --locked
```

For an isolated UI smoke test or performance benchmark, use a **dedicated scratch directory**. These commands generate their own fixture files and reports:

```powershell
target\release\anemorph.exe --smoke-test C:\Temp\Anemorph-smoke
target\release\anemorph.exe --benchmark C:\Temp\Anemorph-benchmark
```

The table renders visible rows only. File scanning, renaming, and large previews run on worker threads; repeated preview edits are coalesced. The interface sleeps when idle and keeps animations short.
