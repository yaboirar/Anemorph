//! Developer verification of the installed COM handler; not shipped in the installer.
use std::{os::windows::ffi::OsStrExt, path::PathBuf};
use windows::{
    Win32::{
        System::Com::{
            CLSCTX_ALL, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx, CoTaskMemFree,
            CoUninitialize,
        },
        UI::Shell::{
            BHID_SFUIObject, Common::ITEMIDLIST, IContextMenu, IExplorerCommand, IShellItemArray,
            SHCreateShellItemArrayFromIDLists, SHParseDisplayName,
        },
        UI::WindowsAndMessaging::{
            CreatePopupMenu, DestroyMenu, GetMenuItemCount, GetMenuStringW, MF_BYPOSITION,
        },
    },
    core::{IUnknown, Interface, PCWSTR, Result},
};
fn run() -> Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
    }
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let invoke = args.first().is_some_and(|s| s == "--invoke");
    let menu = args.first().is_some_and(|s| s == "--menu");
    let classic = args.iter().any(|s| s == "--classic");
    let mut pidls: Vec<*mut ITEMIDLIST> = Vec::new();
    for path in args.iter().skip(usize::from(invoke || menu)) {
        if path == "--classic" {
            continue;
        }
        let path = PathBuf::from(path);
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let mut pidl = std::ptr::null_mut();
        unsafe {
            SHParseDisplayName(PCWSTR(wide.as_ptr()), None, &mut pidl, 0, None)?;
        }
        pidls.push(pidl);
    }
    let pointers: Vec<_> = pidls.iter().map(|p| *p as *const ITEMIDLIST).collect();
    unsafe {
        let items: IShellItemArray = SHCreateShellItemArrayFromIDLists(&pointers)?;
        for pidl in pidls {
            CoTaskMemFree(Some(pidl.cast()));
        }
        let class = if classic {
            windows::core::GUID::from_u128(0x9ca3d8f3_5068_49e3_b792_94c125f1eacc)
        } else {
            anemorph_shell::CLASS_ID
        };
        println!("Activation class: {class:?}; classic mode: {classic}");
        let instance: IUnknown = CoCreateInstance(&class, None, CLSCTX_ALL)?;
        let command: IExplorerCommand = instance.cast()?;
        let title = command.GetTitle(&items)?;
        println!("Title: {}", title.to_string()?);
        CoTaskMemFree(Some(title.0.cast()));
        let state = command.GetState(&items, true)?;
        println!("State: {state}; selected files: {}", items.GetCount()?);
        if menu {
            let context: IContextMenu = items.BindToHandler(None, &BHID_SFUIObject)?;
            let handle = CreatePopupMenu()?;
            context.QueryContextMenu(handle, 0, 1, 0x7fff, 0x1000)?;
            let mut count = 0;
            for index in 0..GetMenuItemCount(handle) {
                let mut buffer = [0u16; 512];
                let length = GetMenuStringW(handle, index as u32, Some(&mut buffer), MF_BYPOSITION);
                let label = String::from_utf16_lossy(&buffer[..length as usize]);
                if label.contains("Anemorph") {
                    println!("Explorer menu entry: {label}");
                    count += 1;
                }
            }
            DestroyMenu(handle)?;
            println!("Explorer menu Anemorph entry count: {count}");
            if count != usize::from(state == 0) {
                return Err(windows::core::Error::new(
                    windows::Win32::Foundation::E_FAIL,
                    "Unexpected Explorer menu visibility",
                ));
            }
        }
        if invoke {
            if classic {
                use windows::Win32::UI::Shell::{IExecuteCommand, IObjectWithSelection};
                use windows::core::Interface;
                command
                    .cast::<IObjectWithSelection>()?
                    .SetSelection(&items)?;
                command.cast::<IExecuteCommand>()?.Execute()?;
            } else {
                command.Invoke(&items, None)?;
            }
            println!("Invoked one selected-files batch");
        }
        drop(command);
        drop(items);
        CoUninitialize();
    }
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
