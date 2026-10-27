//! Trust only the public certificate embedded by the application builder.
use windows_sys::Win32::{Security::Cryptography::*, System::Registry::*};
const CERT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/integration.cer"));
const RECORD: &str = r"Software\Anemorph\IntegrationTrust";
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
pub fn available() -> bool {
    !CERT.is_empty()
}
pub fn change(enable: bool) -> Result<(), String> {
    if !available() {
        return if enable {
            Err("This build has no development certificate. Use a signed setup build.".into())
        } else {
            Ok(())
        };
    }
    unsafe {
        let cert = CertCreateCertificateContext(
            X509_ASN_ENCODING | PKCS_7_ASN_ENCODING,
            CERT.as_ptr(),
            CERT.len() as u32,
        );
        if cert.is_null() {
            return Err("Invalid embedded integration certificate.".into());
        }
        let result = change_inner(cert, enable);
        CertFreeCertificateContext(cert);
        result
    }
}
unsafe fn change_inner(cert: *const CERT_CONTEXT, enable: bool) -> Result<(), String> {
    unsafe {
        let mut hash = [0u8; 20];
        let mut size = 20;
        if CertGetCertificateContextProperty(
            cert,
            CERT_SHA1_HASH_PROP_ID,
            hash.as_mut_ptr().cast(),
            &mut size,
        ) == 0
        {
            return Err("Cannot identify integration certificate.".into());
        }
        let fingerprint = hash.iter().map(|b| format!("{b:02X}")).collect::<String>();
        let mut owned = [0u16; 64];
        let mut len = (owned.len() * 2) as u32;
        let status = RegGetValueW(
            HKEY_LOCAL_MACHINE,
            wide(RECORD).as_ptr(),
            wide("Thumbprint").as_ptr(),
            RRF_RT_REG_SZ | RRF_SUBKEY_WOW6464KEY,
            std::ptr::null_mut(),
            owned.as_mut_ptr().cast(),
            &mut len,
        );
        let ours = status == 0
            && len >= 2
            && String::from_utf16_lossy(&owned[..len as usize / 2 - 1]) == fingerprint;
        if enable && status == 0 && !ours {
            return Err("A different Anemorph signing identity is already registered. Remove it with its original uninstaller first.".into());
        }
        let store = CertOpenStore(
            CERT_STORE_PROV_SYSTEM_W,
            0,
            0,
            CERT_SYSTEM_STORE_LOCAL_MACHINE
                | if enable {
                    0
                } else {
                    CERT_STORE_OPEN_EXISTING_FLAG
                },
            wide("TrustedPeople").as_ptr().cast(),
        );
        if store.is_null() {
            return Err("Cannot open the computer Trusted People store. Administrator approval is required.".into());
        }
        let blob = CRYPT_INTEGER_BLOB {
            cbData: 20,
            pbData: hash.as_mut_ptr(),
        };
        let found = CertFindCertificateInStore(
            store,
            X509_ASN_ENCODING | PKCS_7_ASN_ENCODING,
            0,
            CERT_FIND_SHA1_HASH,
            (&blob as *const CRYPT_INTEGER_BLOB).cast(),
            std::ptr::null(),
        );
        let result = if enable && found.is_null() {
            let mut key = std::ptr::null_mut();
            let code = RegCreateKeyExW(
                HKEY_LOCAL_MACHINE,
                wide(RECORD).as_ptr(),
                0,
                std::ptr::null(),
                0,
                KEY_SET_VALUE | KEY_WOW64_64KEY,
                std::ptr::null(),
                &mut key,
                std::ptr::null_mut(),
            );
            if code != 0 {
                Err("Cannot record Anemorph's certificate ownership.".into())
            } else {
                let text = wide(&fingerprint);
                let code = RegSetValueExW(
                    key,
                    wide("Thumbprint").as_ptr(),
                    0,
                    REG_SZ,
                    text.as_ptr().cast(),
                    (text.len() * 2) as u32,
                );
                RegCloseKey(key);
                if code != 0 {
                    Err("Cannot save certificate ownership.".into())
                } else if CertAddCertificateContextToStore(
                    store,
                    cert,
                    CERT_STORE_ADD_NEW,
                    std::ptr::null_mut(),
                ) == 0
                {
                    RegDeleteKeyExW(
                        HKEY_LOCAL_MACHINE,
                        wide(RECORD).as_ptr(),
                        KEY_WOW64_64KEY,
                        0,
                    );
                    Err("Windows could not trust Anemorph's certificate.".into())
                } else {
                    Ok(())
                }
            }
        } else if !enable && ours {
            if !found.is_null() && CertDeleteCertificateFromStore(found) == 0 {
                CertCloseStore(store, 0);
                return Err("Cannot remove Anemorph's integration certificate.".into());
            }
            RegDeleteKeyExW(
                HKEY_LOCAL_MACHINE,
                wide(RECORD).as_ptr(),
                KEY_WOW64_64KEY,
                0,
            );
            Ok(())
        } else {
            Ok(())
        };
        if !found.is_null() && (enable || !ours) {
            CertFreeCertificateContext(found);
        }
        CertCloseStore(store, 0);
        result
    }
}
pub fn owned() -> bool {
    // A read-only probe lets ordinary uninstall avoid an unnecessary administrator prompt.
    unsafe {
        if !available() {
            return false;
        }
        let cert = CertCreateCertificateContext(
            X509_ASN_ENCODING | PKCS_7_ASN_ENCODING,
            CERT.as_ptr(),
            CERT.len() as u32,
        );
        if cert.is_null() {
            return false;
        }
        let mut hash = [0u8; 20];
        let mut size = 20;
        let read = CertGetCertificateContextProperty(
            cert,
            CERT_SHA1_HASH_PROP_ID,
            hash.as_mut_ptr().cast(),
            &mut size,
        );
        CertFreeCertificateContext(cert);
        if read == 0 {
            return false;
        }
        let expected = hash.iter().map(|b| format!("{b:02X}")).collect::<String>();
        let mut value = [0u16; 64];
        let mut bytes = (value.len() * 2) as u32;
        let code = RegGetValueW(
            HKEY_LOCAL_MACHINE,
            wide(RECORD).as_ptr(),
            wide("Thumbprint").as_ptr(),
            RRF_RT_REG_SZ | RRF_SUBKEY_WOW6464KEY,
            std::ptr::null_mut(),
            value.as_mut_ptr().cast(),
            &mut bytes,
        );
        code == 0
            && bytes >= 2
            && String::from_utf16_lossy(&value[..bytes as usize / 2 - 1]) == expected
    }
}
