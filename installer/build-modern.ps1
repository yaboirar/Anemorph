param(
    [Parameter(Mandatory)][string]$SdkBin,
    [Parameter(Mandatory)][string]$TargetDir,
    [string]$Publisher = 'CN=yaboirar',
    [string]$CertificateThumbprint
)
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$TargetDir = [IO.Path]::GetFullPath($TargetDir)
$stage = Join-Path $TargetDir 'modern-integration'
$layout = Join-Path $stage 'manifest'
New-Item -ItemType Directory -Path $layout -Force | Out-Null
$version = [regex]::Match((Get-Content -LiteralPath (Join-Path $repo 'Cargo.toml') -Raw), '(?m)^version = "([^"]+)"').Groups[1].Value + '.0'
$dll = Join-Path $TargetDir 'release\anemorph_shell.dll'
$hash = (Get-FileHash -LiteralPath $dll -Algorithm SHA256).Hash.Substring(0, 12).ToLowerInvariant()
$xml = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'modern\AppxManifest.xml.in') -Raw
$xml = $xml.Replace('@PUBLISHER@', [Security.SecurityElement]::Escape($Publisher)).Replace('@VERSION@', $version).Replace('@SHELL_DLL@', "anemorph-shell-$hash.dll")
$manifest = Join-Path $layout 'AppxManifest.xml'
[IO.File]::WriteAllText($manifest, $xml, [Text.UTF8Encoding]::new($false))
$package = Join-Path $stage 'Anemorph.identity.msix'
& (Join-Path $SdkBin 'makeappx.exe') pack /o /nv /d $layout /p $package
if ($LASTEXITCODE -ne 0) { throw 'Identity package validation/build failed.' }
if ($CertificateThumbprint) {
    $certificate = Get-Item -LiteralPath ('Cert:\CurrentUser\My\' + $CertificateThumbprint)
    if ($certificate.Subject -ne $Publisher -or -not $certificate.HasPrivateKey) { throw 'Signing certificate does not match the publisher or has no private key.' }
    & (Join-Path $SdkBin 'signtool.exe') sign /fd SHA256 /sha1 $CertificateThumbprint /s My $package
    if ($LASTEXITCODE -ne 0) { throw 'Identity package signing failed.' }
}
Write-Output $package
