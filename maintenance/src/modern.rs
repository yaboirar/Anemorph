//! Per-user identity-package registration for Windows 11's first context menu.
use std::{os::windows::ffi::OsStrExt, path::Path};
use windows::{
    Foundation::Uri,
    Management::Deployment::{AddPackageOptions, DeploymentResult, PackageManager},
    Win32::{
        System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize, RoUninitialize},
        UI::Shell::UrlCreateFromPathW,
    },
    core::{HSTRING, PCWSTR, PWSTR},
};
const NAME: &str = "yaboirar.Anemorph";
struct Runtime;
impl Runtime {
    fn new() -> Result<Self, String> {
        unsafe { RoInitialize(RO_INIT_MULTITHREADED) }.map_err(|e| e.to_string())?;
        Ok(Self)
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        unsafe {
            RoUninitialize();
        }
    }
}
fn uri(path: &Path) -> Result<Uri, String> {
    if !path.is_absolute() {
        return Err("The integration location must be absolute.".into());
    }
    let input: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut output = vec![0u16; 65536];
    let mut size = output.len() as u32;
    unsafe {
        UrlCreateFromPathW(
            PCWSTR(input.as_ptr()),
            PWSTR(output.as_mut_ptr()),
            &mut size,
            0,
        )
    }
    .map_err(|e| e.to_string())?;
    let end = output
        .iter()
        .position(|c| *c == 0)
        .ok_or("Invalid integration location URI")?;
    Uri::CreateUri(&HSTRING::from_wide(&output[..end]).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}
fn completed(result: DeploymentResult) -> Result<(), String> {
    let code = result.ExtendedErrorCode().map_err(|e| e.to_string())?;
    if code.is_err() {
        let text = result
            .ErrorText()
            .map(|s| s.to_string())
            .unwrap_or_default();
        return Err(format!(
            "Windows package registration failed ({code:?}): {text}"
        ));
    }
    Ok(())
}
pub fn register(root: &Path) -> Result<(), String> {
    let _runtime = Runtime::new()?;
    let package = root.join("Anemorph.Integration/Anemorph.identity.msix");
    if !root.is_absolute() || !package.is_file() {
        return Err("The signed Explorer integration package is missing.".into());
    }
    let manager = PackageManager::new().map_err(|e| e.to_string())?;
    let options = AddPackageOptions::new().map_err(|e| e.to_string())?;
    options
        .SetExternalLocationUri(&uri(root)?)
        .map_err(|e| e.to_string())?;
    options
        .SetForceUpdateFromAnyVersion(true)
        .map_err(|e| e.to_string())?;
    // Windows performs signature and trust validation. No trust store or policy is changed here.
    let result = manager
        .AddPackageByUriAsync(&uri(&package)?, &options)
        .map_err(|e| e.to_string())?
        .get()
        .map_err(|e| e.to_string())?;
    completed(result)?;
    crate::broker_control::stop(root)?;
    crate::menu::packaged()?;
    println!("Windows 11 context-menu integration registered.");
    Ok(())
}
pub fn unregister() -> Result<(), String> {
    let _runtime = Runtime::new()?;
    let manager = PackageManager::new().map_err(|e| e.to_string())?;
    let packages = manager
        .FindPackagesByUserSecurityId(&HSTRING::new())
        .map_err(|e| e.to_string())?;
    let mut owned = Vec::new();
    for package in packages {
        let id = package.Id().map_err(|e| e.to_string())?;
        if id.Name().map_err(|e| e.to_string())? == NAME {
            owned.push(id.FullName().map_err(|e| e.to_string())?);
        }
    }
    for name in owned {
        completed(
            manager
                .RemovePackageAsync(&name)
                .map_err(|e| e.to_string())?
                .get()
                .map_err(|e| e.to_string())?,
        )?;
    }
    Ok(())
}
pub fn enabled() -> Result<bool, String> {
    let _runtime = Runtime::new()?;
    let manager = PackageManager::new().map_err(|e| e.to_string())?;
    for package in manager
        .FindPackagesByUserSecurityId(&HSTRING::new())
        .map_err(|e| e.to_string())?
    {
        if package
            .Id()
            .and_then(|id| id.Name())
            .map_err(|e| e.to_string())?
            == NAME
        {
            return Ok(true);
        }
    }
    Ok(false)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn file_uri_preserves_spaces_and_special_path_characters() {
        let _runtime = Runtime::new().unwrap();
        let value = uri(Path::new(r"C:\Anemorph space\A#B\context.msix")).unwrap();
        let path = value.Path().unwrap().to_string();
        let fragment = value.Fragment().unwrap().to_string();
        assert!(path.replace("%20", " ").contains("Anemorph space"));
        assert!(
            path.contains("A#B") && fragment.is_empty(),
            "path={path}; fragment={fragment}"
        );
        assert!(uri(Path::new("relative.msix")).is_err());
    }
}
