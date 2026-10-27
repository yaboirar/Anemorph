//! Remove only the explicitly recorded, Anemorph-created local development identity.
use windows_sys::Win32::{
    Foundation::{
        CRYPT_E_NOT_FOUND, ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND, GetLastError,
        NTE_BAD_KEYSET, NTE_NOT_FOUND,
    },
    Security::Cryptography::*,
    System::Registry::*,
};
fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
fn setting(name: &str) -> Option<String> {
    let key = wide(r"Software\Anemorph\LocalSigning");
    let name = wide(name);
    let mut value = [0u16; 256];
    let mut size = (value.len() * 2) as u32;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            name.as_ptr(),
            RRF_RT_REG_SZ | RRF_SUBKEY_WOW6464KEY,
            std::ptr::null_mut(),
            value.as_mut_ptr().cast(),
            &mut size,
        )
    };
    if status != 0 || size < 2 {
        return None;
    }
    Some(String::from_utf16_lossy(&value[..size as usize / 2 - 1]))
}
fn fingerprint(value: &str) -> Result<[u8; 20], String> {
    if value.len() != 40 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Invalid recorded local certificate fingerprint.".into());
    }
    let mut bytes = [0u8; 20];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte =
            u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).map_err(|e| e.to_string())?;
    }
    Ok(bytes)
}
fn remove_certificate(store_name: &str, hash: &mut [u8; 20]) -> Result<(), String> {
    let name = wide(store_name);
    unsafe {
        let store = CertOpenStore(
            CERT_STORE_PROV_SYSTEM_W,
            0,
            0,
            CERT_SYSTEM_STORE_CURRENT_USER | CERT_STORE_OPEN_EXISTING_FLAG,
            name.as_ptr().cast(),
        );
        if store.is_null() {
            let code = GetLastError();
            if code == ERROR_FILE_NOT_FOUND
                || code == ERROR_PATH_NOT_FOUND
                || code == CRYPT_E_NOT_FOUND as u32
            {
                return Ok(());
            }
            return Err(format!(
                "Could not open current-user {store_name} certificate store."
            ));
        }
        let blob = CRYPT_INTEGER_BLOB {
            cbData: hash.len() as u32,
            pbData: hash.as_mut_ptr(),
        };
        let certificate = CertFindCertificateInStore(
            store,
            X509_ASN_ENCODING | PKCS_7_ASN_ENCODING,
            0,
            CERT_FIND_SHA1_HASH,
            (&blob as *const CRYPT_INTEGER_BLOB).cast(),
            std::ptr::null(),
        );
        let result = if certificate.is_null() {
            let code = GetLastError();
            if code == CRYPT_E_NOT_FOUND as u32 {
                Ok(())
            } else {
                Err(format!("Certificate lookup failed: Windows error {code}"))
            }
        } else if CertDeleteCertificateFromStore(certificate) != 0 {
            Ok(())
        } else {
            Err(format!(
                "Could not remove Anemorph's certificate from {store_name}."
            ))
        };
        CertCloseStore(store, 0);
        result
    }
}
fn remove_key(name: &str) -> Result<(), String> {
    if !name.starts_with("Anemorph.LocalSigning.")
        || !name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'.' || c == b'-')
    {
        return Err("Invalid recorded Anemorph signing-key name.".into());
    }
    unsafe {
        let mut provider = 0;
        let status = NCryptOpenStorageProvider(
            &mut provider,
            wide("Microsoft Software Key Storage Provider").as_ptr(),
            0,
        );
        if status != 0 {
            return Err(format!("Signing-key provider error: {status:#x}"));
        }
        let mut key = 0;
        let status = NCryptOpenKey(
            provider,
            &mut key,
            wide(name).as_ptr(),
            0,
            NCRYPT_SILENT_FLAG,
        );
        let result = if status == NTE_BAD_KEYSET || status == NTE_NOT_FOUND {
            Ok(())
        } else if status != 0 {
            Err(format!("Could not open the local signing key: {status:#x}"))
        } else {
            let status = NCryptDeleteKey(key, NCRYPT_SILENT_FLAG);
            if status == 0 {
                Ok(())
            } else {
                NCryptFreeObject(key);
                Err(format!(
                    "Could not remove the local signing key: {status:#x}"
                ))
            }
        };
        NCryptFreeObject(provider);
        result
    }
}
pub fn remove() -> Result<(), String> {
    if setting("Owned").as_deref() != Some("1") {
        return Ok(());
    }
    let mut hash =
        fingerprint(&setting("Thumbprint").ok_or("Local certificate fingerprint is missing.")?)?;
    let key = setting("KeyName").ok_or("Local signing-key ownership record is missing.")?;
    remove_key(&key)?;
    remove_certificate("TrustedPeople", &mut hash)?;
    remove_certificate("My", &mut hash)?;
    println!("Removed Anemorph's local test certificate and signing key for this Windows account.");
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_malformed_ownership_records_without_deleting_anything() {
        assert!(fingerprint("invalid").is_err());
        assert!(remove_key("OtherApplicationKey").is_err());
        assert_eq!(fingerprint(&"AB".repeat(20)).unwrap(), [0xAB; 20]);
    }
}
