# Trusted publisher signing for direct downloads

The unknown-publisher and SmartScreen messages belong to Windows. Anemorph cannot suppress them for other users. A local test certificate does not establish trust on a different PC. Do not disable SmartScreen or ask users to install a private root certificate.

For direct GitHub downloads, obtain a publicly trusted code-signing identity from a certificate authority or verified signing provider. This requires the publisher's identity check. New signed downloads may still receive a SmartScreen reputation warning; a signature is not a promise that every Windows warning disappears.

## Release build support

`installer/sign.ps1` validates code-signing trust and revocation, rejects self-signed test certificates, signs with SHA-256 and a timestamp, and verifies the result. Private keys remain in the certificate store or its hardware provider and are never exported into the repository or downloads.

With a CA-issued certificate and its private key available in CurrentUser/My, build the Windows menu identity using the matching publisher subject and certificate. Then build the installer with:

```powershell
.\installer\build.ps1 -TargetDir target -OutputDir dist -ReleaseCertificateThumbprint "YOUR_VERIFIED_CERTIFICATE_THUMBPRINT" -SignTool "C:\Windows SDK\signtool.exe"
```

The signed build signs the app, menu host, maintenance helper, and generated uninstaller before packing them, then signs the final installer. It refuses to mix a production certificate with a menu identity signed by a different test certificate. Generate download checksums only after signing.

Publisher identity is the legal name approved by the signing provider. It cannot be replaced by inventing a publisher label in the source code. Cloud providers with their own signing tools need their corresponding signing hook; this certificate-store path does not provision a cloud account.

## Current limit

The existing Anemorph certificate is self-signed for development. No publicly trusted publisher certificate or verified signing-service account has been supplied. The build support is ready, but production signing and removal of the unknown-publisher label remain pending that identity. This document does not claim that an unsigned build is warning-free.

Sources: [Microsoft signing options](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options), [SmartScreen reputation](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation).

## Free open-source signing route

SignPath Foundation offers free signing to approved open-source projects: https://signpath.org/apply. Its review requirements include open-source components, maintainer/repository verification, two-factor authentication, verifiable builds, and release approval. Approval is external and not guaranteed.

Anemorph's supplied Anemo artwork currently has no established redistribution or open-source license. It is excluded from MIT in THIRD_PARTY_NOTICES.md. This is a concrete eligibility concern for the free program and must be disclosed, not silently declared compliant or replaced without the maintainer's authorization. A local application draft is prepared, but no application has been submitted and no sponsorship is claimed.

No purchase or paid subscription has been authorized. The existing certificate-store signing hook is build support, not an instruction to buy a certificate.
