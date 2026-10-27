//! Small Explorer command: no scanning, renaming, network access or UI in Explorer.
#![allow(non_snake_case)]
#[allow(dead_code)]
#[path = "../../src/containers.rs"]
mod containers;
#[allow(dead_code)]
#[path = "../../src/selection.rs"]
mod selection;
use std::{
    cell::RefCell,
    ffi::{OsString, c_void},
    os::windows::ffi::OsStringExt,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
use windows::{
    Win32::{
        Foundation::{
            BOOL, CLASS_E_CLASSNOTAVAILABLE, CLASS_E_NOAGGREGATION, E_FAIL, E_INVALIDARG,
            E_NOTIMPL, E_POINTER, HMODULE, POINT, S_FALSE, S_OK,
        },
        System::{
            Com::{CoTaskMemFree, IBindCtx, IClassFactory, IClassFactory_Impl},
            LibraryLoader::{
                GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
                GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT, GetModuleFileNameW,
                GetModuleHandleExW,
            },
        },
        UI::Shell::{
            ECS_ENABLED, ECS_HIDDEN, IEnumExplorerCommand, IExecuteCommand, IExecuteCommand_Impl,
            IExplorerCommand, IExplorerCommand_Impl, IObjectWithSelection,
            IObjectWithSelection_Impl, IShellItemArray, SHStrDupW, SIGDN_FILESYSPATH,
        },
    },
    core::{Error, GUID, HRESULT, IUnknown, Interface, PCWSTR, PWSTR, Result, implement},
};
pub const CLASS_ID: GUID = GUID::from_u128(0xb8a9bdb0_03e4_4f1a_bdea_28654a8f7db4);
static OBJECTS: AtomicUsize = AtomicUsize::new(0);
static LOCKS: AtomicUsize = AtomicUsize::new(0);
struct Live;
impl Live {
    fn new() -> Self {
        OBJECTS.fetch_add(1, Ordering::SeqCst);
        Self
    }
}
impl Drop for Live {
    fn drop(&mut self) {
        OBJECTS.fetch_sub(1, Ordering::SeqCst);
    }
}

fn executable() -> Result<PathBuf> {
    let mut module = HMODULE::default();
    // The address belongs to this DLL; UNCHANGED_REFCOUNT avoids leaking a module reference.
    unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            PCWSTR(DllGetClassObject as *const () as *const u16),
            &mut module,
        )?;
    }
    let mut buffer = vec![0u16; 32768];
    let len = unsafe { GetModuleFileNameW(module, &mut buffer) } as usize;
    if len == 0 || len >= buffer.len() {
        return Err(Error::from_win32());
    }
    let dll = PathBuf::from(OsString::from_wide(&buffer[..len]));
    Ok(dll
        .parent()
        .ok_or_else(|| Error::from(E_FAIL))?
        .join("Anemorph.exe"))
}
fn paths(items: Option<&IShellItemArray>) -> Result<Vec<PathBuf>> {
    let items = items.ok_or_else(|| Error::from(E_INVALIDARG))?;
    // Shell owns the array and keeps it alive for the call. Every returned string is freed.
    unsafe {
        let count = items.GetCount()?;
        if count == 0 || count as usize > selection::MAX_FILES {
            return Err(Error::from(E_INVALIDARG));
        }
        let mut files = Vec::with_capacity(count as usize);
        for index in 0..count {
            let name = items.GetItemAt(index)?.GetDisplayName(SIGDN_FILESYSPATH)?;
            let path = PathBuf::from(OsString::from_wide(name.as_wide()));
            CoTaskMemFree(Some(name.0.cast()));
            if !selection::supported(&path) || !path.is_absolute() {
                return Err(Error::from(E_INVALIDARG));
            }
            files.push(path);
        }
        Ok(files)
    }
}
fn text(value: &str) -> Result<PWSTR> {
    let wide: Vec<u16> = value.encode_utf16().chain(Some(0)).collect();
    // Explorer takes ownership of the COM-allocated copy.
    unsafe { SHStrDupW(PCWSTR(wide.as_ptr())) }
}
#[implement(IExplorerCommand, IExecuteCommand, IObjectWithSelection)]
struct Command {
    _live: Live,
    selection: RefCell<Option<IShellItemArray>>,
}
impl IObjectWithSelection_Impl for Command_Impl {
    fn SetSelection(&self, items: Option<&IShellItemArray>) -> Result<()> {
        paths(items)?;
        *self.selection.borrow_mut() = items.cloned();
        Ok(())
    }
    fn GetSelection(&self, iid: *const GUID, out: *mut *mut c_void) -> Result<()> {
        if iid.is_null() || out.is_null() {
            return Err(Error::from(E_POINTER));
        }
        unsafe {
            *out = std::ptr::null_mut();
        }
        let items = self.selection.borrow();
        let items = items.as_ref().ok_or_else(|| Error::from(E_INVALIDARG))?;
        unsafe { items.query(iid, out).ok() }
    }
}
impl IExecuteCommand_Impl for Command_Impl {
    fn SetKeyState(&self, _: u32) -> Result<()> {
        Ok(())
    }
    fn SetParameters(&self, _: &PCWSTR) -> Result<()> {
        Ok(())
    }
    fn SetPosition(&self, _: &POINT) -> Result<()> {
        Ok(())
    }
    fn SetShowWindow(&self, _: i32) -> Result<()> {
        Ok(())
    }
    fn SetNoShowUI(&self, _: BOOL) -> Result<()> {
        Ok(())
    }
    fn SetDirectory(&self, _: &PCWSTR) -> Result<()> {
        Ok(())
    }
    fn Execute(&self) -> Result<()> {
        self.Invoke(self.selection.borrow().as_ref(), None)
    }
}
impl IExplorerCommand_Impl for Command_Impl {
    fn GetTitle(&self, _: Option<&IShellItemArray>) -> Result<PWSTR> {
        text("Rename with Anemorph")
    }
    fn GetIcon(&self, _: Option<&IShellItemArray>) -> Result<PWSTR> {
        text(&format!("{},0", executable()?.display()))
    }
    fn GetToolTip(&self, _: Option<&IShellItemArray>) -> Result<PWSTR> {
        text("Preview names for the selected videos")
    }
    fn GetCanonicalName(&self) -> Result<GUID> {
        Ok(CLASS_ID)
    }
    fn GetState(&self, items: Option<&IShellItemArray>, _: BOOL) -> Result<u32> {
        Ok(if paths(items).is_ok() {
            ECS_ENABLED.0 as u32
        } else {
            ECS_HIDDEN.0 as u32
        })
    }
    fn Invoke(&self, items: Option<&IShellItemArray>, _: Option<&IBindCtx>) -> Result<()> {
        let files = paths(items)?;
        let id = unsafe { windows::Win32::System::Com::CoCreateGuid()? };
        let request = selection::write_request(&selection::directory(), &format!("{id:?}"), &files)
            .map_err(|e| Error::new(E_FAIL, e))?;
        // Pass a single manifest argument, avoiding command-line limits and shell interpolation.
        let result = std::process::Command::new(executable()?)
            .arg("--selection")
            .arg(&request)
            .spawn();
        if let Err(error) = result {
            let _ = std::fs::remove_file(request);
            return Err(Error::new(E_FAIL, error.to_string()));
        }
        Ok(())
    }
    fn GetFlags(&self) -> Result<u32> {
        Ok(0)
    }
    fn EnumSubCommands(&self) -> Result<IEnumExplorerCommand> {
        Err(Error::from(E_NOTIMPL))
    }
}
#[implement(IClassFactory)]
struct Factory {
    _live: Live,
}
impl IClassFactory_Impl for Factory_Impl {
    fn CreateInstance(
        &self,
        outer: Option<&IUnknown>,
        iid: *const GUID,
        out: *mut *mut c_void,
    ) -> Result<()> {
        if out.is_null() || iid.is_null() {
            return Err(Error::from(E_POINTER));
        }
        unsafe {
            *out = std::ptr::null_mut();
        }
        if outer.is_some() {
            return Err(Error::from(CLASS_E_NOAGGREGATION));
        }
        let command: IExplorerCommand = Command {
            _live: Live::new(),
            selection: RefCell::new(None),
        }
        .into();
        // COM supplies valid writable out and IID pointers; query adds the returned reference.
        unsafe { command.query(iid, out).ok() }
    }
    fn LockServer(&self, lock: BOOL) -> Result<()> {
        if lock.as_bool() {
            LOCKS.fetch_add(1, Ordering::SeqCst);
        } else {
            let _ = LOCKS.try_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_sub(1));
        }
        Ok(())
    }
}
pub fn class_factory() -> IClassFactory {
    Factory { _live: Live::new() }.into()
}
pub fn server_idle() -> bool {
    OBJECTS.load(Ordering::SeqCst) <= 1 && LOCKS.load(Ordering::SeqCst) == 0
}
/// # Safety
/// Called by COM with valid IID/CLSID and writable output pointers.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllGetClassObject(
    class: *const GUID,
    iid: *const GUID,
    out: *mut *mut c_void,
) -> HRESULT {
    if out.is_null() || class.is_null() || iid.is_null() {
        return E_POINTER;
    }
    unsafe {
        *out = std::ptr::null_mut();
    }
    if unsafe { *class } != CLASS_ID {
        return CLASS_E_CLASSNOTAVAILABLE;
    }
    let factory: IClassFactory = Factory { _live: Live::new() }.into();
    unsafe { factory.query(iid, out) }
}
#[unsafe(no_mangle)]
pub extern "system" fn DllCanUnloadNow() -> HRESULT {
    if OBJECTS.load(Ordering::SeqCst) == 0 && LOCKS.load(Ordering::SeqCst) == 0 {
        S_OK
    } else {
        S_FALSE
    }
}
