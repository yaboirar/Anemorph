<div align="center">

# Release signing

**What Windows trust means for Anemorph downloads.**

[Home](../README.md) · [Build from source](building.md) · [Third-party notices](licenses/third-party-notices.md)

</div>

---

## Current status

**The public installer is unsigned.** Windows may show an unknown-publisher or SmartScreen warning. The development certificate used for the optional Windows 11 menu does not establish publisher trust on other PCs.

| Item | What it provides |
| :--- | :--- |
| Development certificate | Local testing where a user explicitly approves trust |
| Verified release signature | A verified publisher identity and a check that signed files were not altered |
| SmartScreen reputation | A Windows decision; new signed downloads can still receive a warning |

A signature does not guarantee that every Windows warning disappears. Anemorph does not disable SmartScreen or weaken Windows security policy.

---

## Sign a release

The build supports a publicly trusted code-signing certificate and private key available in **CurrentUser/My**. Build the Windows menu identity with the same publisher and certificate, then run:

```powershell
.\installer\build.ps1 -TargetDir target -OutputDir dist -ReleaseCertificateThumbprint "YOUR_VERIFIED_CERTIFICATE_THUMBPRINT" -SignTool "C:\Windows SDK\signtool.exe"
```

| Step | What the build does |
| :--- | :--- |
| Validate | Check certificate trust, revocation, and code-signing use; reject self-signed test certificates |
| Sign app files | Sign the app, menu host, shared library, and maintenance helper |
| Package | Generate and sign the uninstaller, then build the installer |
| Finish | Sign and verify the final installer using SHA-256 and a timestamp |

The build refuses a menu identity signed by a different certificate. Generate download checksums **after** signing.

Private keys stay in the certificate store or its hardware provider. Never export them into the repository or release downloads. Cloud signing services need their own integration; the certificate-store build option does not provision a service account.

The publisher name comes from the identity approved by the signing provider. Changing a label in source code cannot create a verified publisher.

---

## Free signing for open-source projects

[SignPath Foundation](https://signpath.org/apply) accepts applications for free signing. Approval is external and not guaranteed. Its [terms](https://signpath.org/terms) include open-source components, project verification, multi-factor authentication, verifiable builds, and release approval.

**The supplied Anemo artwork needs review before claiming eligibility.** It is excluded from MIT, and redistribution permission has not been established. See [third-party notices](licenses/third-party-notices.md). No SignPath approval or sponsorship is claimed.

---

## References

- [Microsoft: signing options](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options)
- [Microsoft: SmartScreen reputation](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation)
- [SignPath Foundation: application](https://signpath.org/apply) and [terms](https://signpath.org/terms)

---

[Back to the build guide](building.md) · [Back to Anemorph](../README.md)
