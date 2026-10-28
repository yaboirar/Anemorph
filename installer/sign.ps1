param(
    [Parameter(Mandatory)][ValidatePattern('^[A-Fa-f0-9]{40}$')][string]$CertificateThumbprint,
    [string[]]$Files,
    [string]$SignTool,
    [string]$TimestampServer = 'http://timestamp.digicert.com',
    [switch]$ValidateOnly
)
$ErrorActionPreference = 'Stop'
$certificate = Get-Item -LiteralPath ('Cert:\CurrentUser\My\' + $CertificateThumbprint)
if (-not $certificate.HasPrivateKey) { throw 'The verified publisher certificate needs its private key on this build machine.' }
if ($certificate.Subject -eq $certificate.Issuer) { throw 'A self-signed test certificate cannot identify a verified publisher on other PCs. Use a CA-issued code-signing certificate.' }
$chain = [Security.Cryptography.X509Certificates.X509Chain]::new()
try {
    $chain.ChainPolicy.RevocationMode = [Security.Cryptography.X509Certificates.X509RevocationMode]::Online
    $chain.ChainPolicy.VerificationFlags = [Security.Cryptography.X509Certificates.X509VerificationFlags]::NoFlag
    $chain.ChainPolicy.ApplicationPolicy.Add([Security.Cryptography.Oid]::new('1.3.6.1.5.5.7.3.3'))
    if (-not $chain.Build($certificate)) { throw 'The publisher certificate failed Windows code-signing trust or revocation checks.' }
} finally { $chain.Dispose() }
if ($ValidateOnly) { Write-Output 'Publisher certificate passed local validation.'; return }
if (-not $Files) { throw 'No release files were supplied for signing.' }
if (-not $SignTool) { $SignTool = (Get-Command signtool.exe -ErrorAction Stop).Source }
$SignTool = (Resolve-Path -LiteralPath $SignTool).Path
foreach ($file in $Files) {
    $path = (Resolve-Path -LiteralPath $file).Path
    if ([IO.Path]::GetExtension($path).ToLowerInvariant() -notin @('.exe','.dll','.msix')) { throw 'Only Windows release binaries and packages can be signed.' }
    & $SignTool sign /fd SHA256 /tr $TimestampServer /td SHA256 /sha1 $CertificateThumbprint /s My $path
    if ($LASTEXITCODE -ne 0) { throw ('Publisher signing failed for ' + [IO.Path]::GetFileName($path)) }
    & $SignTool verify /pa /all $path
    if ($LASTEXITCODE -ne 0) { throw ('Windows signature verification failed for ' + [IO.Path]::GetFileName($path)) }
    $signed = Get-AuthenticodeSignature -LiteralPath $path
    if ($signed.Status -ne 'Valid' -or $signed.SignerCertificate.Thumbprint -ne $CertificateThumbprint) { throw 'The resulting signature does not match the verified publisher.' }
}
