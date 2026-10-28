param([string]$TargetDir, [string]$OutputDir, [switch]$SkipBuild, [string]$ModernPackage, [string]$ReleaseCertificateThumbprint, [string]$SignTool)
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
if (-not $TargetDir) { $TargetDir = Join-Path $repo 'target' }
if (-not $OutputDir) { $OutputDir = Join-Path $repo 'dist' }
$TargetDir = [IO.Path]::GetFullPath($TargetDir)
$OutputDir = [IO.Path]::GetFullPath($OutputDir)
$env:CARGO_TARGET_DIR = $TargetDir
Push-Location $repo
try {
    if (-not $SkipBuild) {
        & cargo build --release --locked --workspace
        if ($LASTEXITCODE -ne 0) { throw 'Rust release build failed.' }
    }
    $version = [regex]::Match((Get-Content -LiteralPath Cargo.toml -Raw), '(?m)^version = "([^"]+)"').Groups[1].Value
    $app = Join-Path $TargetDir 'release\anemorph.exe'
    $shell = Join-Path $TargetDir 'release\anemorph_shell.dll'
    $maintenance = Join-Path $TargetDir 'release\anemorph-maintenance.exe'
    $broker = Join-Path $TargetDir 'release\anemorph-broker.exe'
    $signArgs = @()
    if ($ReleaseCertificateThumbprint) {
        & (Join-Path $PSScriptRoot 'sign.ps1') -CertificateThumbprint $ReleaseCertificateThumbprint -ValidateOnly
        if (-not $SignTool) { $SignTool = (Get-Command signtool.exe -ErrorAction Stop).Source }
        $SignTool = (Resolve-Path -LiteralPath $SignTool).Path
        & (Join-Path $PSScriptRoot 'sign.ps1') -CertificateThumbprint $ReleaseCertificateThumbprint -SignTool $SignTool -Files @($app,$maintenance,$broker,$shell)
        $signArgs = @("/DRELEASE_CERT=$ReleaseCertificateThumbprint", "/DSIGN_TOOL=$SignTool", "/DSIGN_SCRIPT=$(Join-Path $PSScriptRoot 'sign.ps1')", "/DSIGN_POWERSHELL=$((Get-Command powershell.exe).Source)")
    }
    $hash = (Get-FileHash -LiteralPath $shell -Algorithm SHA256).Hash.Substring(0, 12).ToLowerInvariant()
    $generated = Join-Path $TargetDir 'installer'
    New-Item -ItemType Directory -Path $generated, $OutputDir -Force | Out-Null
    $register = Join-Path $generated 'register.nsh'
    $unregister = Join-Path $generated 'unregister.nsh'
    $containers = [regex]::Matches((Get-Content -LiteralPath src\containers.rs -Raw), '\("([a-z0-9]+)", "[^"]+"\)') | ForEach-Object { $_.Groups[1].Value }
    if ($containers.Count -ne 26) { throw 'Unexpected container list; review installer registration.' }
    $containers | ForEach-Object { '!insertmacro RegisterVideo ' + $_ } | Set-Content -LiteralPath $register -Encoding utf8
    $containers | ForEach-Object { '!insertmacro UnregisterVideo ' + $_ } | Set-Content -LiteralPath $unregister -Encoding utf8
    $output = Join-Path $OutputDir "Anemorph-Setup-$version.exe"
    $kb = [math]::Ceiling(((Get-Item $app).Length + (Get-Item $shell).Length + (Get-Item docs/licenses/dependency-licenses.md).Length) / 1024)
    if (-not $ModernPackage) {
        $cachedIdentity = Join-Path $TargetDir 'modern-integration\Anemorph.identity.msix'
        if ((Test-Path -LiteralPath $cachedIdentity) -and (Get-AuthenticodeSignature -LiteralPath $cachedIdentity).SignerCertificate) {
            $ModernPackage = $cachedIdentity
        }
    }
    $modernArgs = @()
    if ($ModernPackage) {
        $ModernPackage = [IO.Path]::GetFullPath($ModernPackage)
        if (-not (Test-Path -LiteralPath $ModernPackage)) { throw 'Modern integration package is missing.' }
        if ($ReleaseCertificateThumbprint -and (Get-AuthenticodeSignature -LiteralPath $ModernPackage).SignerCertificate.Thumbprint -ne $ReleaseCertificateThumbprint) { throw 'Rebuild the Windows menu package using the same verified publisher certificate before a signed release.' }
        if (-not (Get-AuthenticodeSignature -LiteralPath $ModernPackage).SignerCertificate) { throw 'The modern integration package must be signed before it is bundled for installation.' }
        $zip = [IO.Compression.ZipFile]::OpenRead($ModernPackage)
        try {
            $reader = [IO.StreamReader]::new($zip.GetEntry('AppxManifest.xml').Open())
            try { [xml]$manifest = $reader.ReadToEnd() } finally { $reader.Dispose() }
        } finally { $zip.Dispose() }
        $class = $manifest.SelectSingleNode("//*[local-name()='Class' and @Id='B8A9BDB0-03E4-4F1A-BDEA-28654A8F7DB4']")
        if (-not $class -or $class.ParentNode.LocalName -ne 'ExeServer' -or $class.ParentNode.Executable -ne 'anemorph-broker.exe' -or $manifest.Package.Identity.Name -ne 'yaboirar.Anemorph' -or $manifest.Package.Identity.Version -ne ($version + '.0')) {
            throw 'Rebuild the identity package: its version or menu host does not match this application build.'
        }
        $modernArgs = @("/DMODERN_PACKAGE=$ModernPackage")
    }
    & makensis.exe /V2 "/DAPP_VERSION=$version" "/DAPP_EXE=$app" "/DMAINTENANCE_EXE=$maintenance" "/DBROKER_EXE=$broker" "/DSHELL_DLL=$shell" "/DSHELL_NAME=anemorph-shell-$hash.dll" "/DOUTPUT_FILE=$output" "/DREGISTER_INCLUDE=$register" "/DUNREGISTER_INCLUDE=$unregister" "/DINSTALLED_KB=$kb" @modernArgs @signArgs (Join-Path $PSScriptRoot 'Anemorph.nsi')
    if ($LASTEXITCODE -ne 0) { throw 'Installer build failed.' }
    if ($ReleaseCertificateThumbprint) {
        & (Join-Path $PSScriptRoot 'sign.ps1') -CertificateThumbprint $ReleaseCertificateThumbprint -SignTool $SignTool -Files @($output)
    }
    Get-Item -LiteralPath $output
} finally { Pop-Location }
