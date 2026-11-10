#[cfg(windows)]
use windows_sys::Win32::{
    Foundation::{HWND, POINT, RECT},
    UI::WindowsAndMessaging::{
        GetCursorPos, GetWindowRect, IsZoomed, SW_RESTORE, SWP_NOACTIVATE, SWP_NOSIZE,
        SWP_NOZORDER, SetWindowPos, ShowWindow,
    },
};

/// Borderless Windows windows do not reliably restore themselves on caption drag.
/// Restore synchronously, anchor the normal-sized window under the pointer, then
/// start the Windows move operation using correctly packed screen coordinates.
#[cfg(windows)]
pub fn begin(handle: usize, title_height: f32) {
    let hwnd = handle as HWND;
    let mut cursor = POINT { x: 0, y: 0 };
    // Safety: the HWND is borrowed from this app's live creation context. These
    // synchronous calls retain no pointers to our stack-owned coordinates.
    unsafe {
        if GetCursorPos(&mut cursor) == 0 {
            return;
        }
    }
    unsafe {
        use windows_sys::Win32::UI::{
            Input::KeyboardAndMouse::ReleaseCapture,
            Shell::SetWindowSubclass,
            WindowsAndMessaging::{HTCAPTION, PostMessageW, WM_NCLBUTTONDOWN},
        };
        if SetWindowSubclass(hwnd, Some(drag_messages), 0x414e4447, 0) == 0 {
            return;
        }
        if IsZoomed(hwnd) != 0 && !restore_at(handle, cursor.x, cursor.y, title_height) {
            return;
        }
        ReleaseCapture();
        // WM_NCLBUTTONDOWN takes signed screen coordinates packed in LPARAM.
        // winit 0.30's Windows path passes a stack address here instead.
        PostMessageW(
            hwnd,
            WM_NCLBUTTONDOWN,
            HTCAPTION as usize,
            packed_point(cursor.x, cursor.y),
        );
    }
}
#[cfg(windows)]
unsafe extern "system" fn drag_messages(
    hwnd: HWND,
    message: u32,
    w: usize,
    l: isize,
    id: usize,
    _data: usize,
) -> isize {
    use windows_sys::Win32::UI::{
        Shell::{DefSubclassProc, RemoveWindowSubclass},
        WindowsAndMessaging::{PostMessageW, WM_EXITSIZEMOVE, WM_LBUTTONUP, WM_NCDESTROY},
    };
    unsafe {
        if message == WM_EXITSIZEMOVE {
            // Our native move bypasses winit's internal dragging flag. Release
            // egui's client-area press after Windows consumes the native mouse-up.
            PostMessageW(hwnd, WM_LBUTTONUP, 0, 0);
        }
        if message == WM_NCDESTROY {
            RemoveWindowSubclass(hwnd, Some(drag_messages), id);
        }
        DefSubclassProc(hwnd, message, w, l)
    }
}
#[cfg(windows)]
fn packed_point(x: i32, y: i32) -> isize {
    ((x as u16 as u32) | ((y as u16 as u32) << 16)) as isize
}
#[cfg(not(windows))]
pub fn begin(_handle: usize, _title_height: f32) {}

#[cfg(windows)]
pub fn restore_at(handle: usize, cursor_x: i32, cursor_y: i32, title_height: f32) -> bool {
    let hwnd = handle as HWND;
    let mut before = RECT::default();
    let mut after = RECT::default();
    // Safety: callers supply this app's live HWND, and all RECT pointers are valid
    // for the duration of these synchronous Windows calls on the UI thread.
    unsafe {
        if IsZoomed(hwnd) == 0 || GetWindowRect(hwnd, &mut before) == 0 {
            return false;
        }
        ShowWindow(hwnd, SW_RESTORE);
        if IsZoomed(hwnd) != 0 || GetWindowRect(hwnd, &mut after) == 0 {
            return false;
        }
        let (x, y) = anchored_position(
            [before.left, before.top, before.right, before.bottom],
            after.right - after.left,
            cursor_x,
            cursor_y,
            title_height,
        );
        SetWindowPos(
            hwnd,
            std::ptr::null_mut(),
            x,
            y,
            0,
            0,
            SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        ) != 0
    }
}

#[cfg(any(windows, test))]
fn anchored_position(
    maximized: [i32; 4],
    restored_width: i32,
    x: i32,
    y: i32,
    title_height: f32,
) -> (i32, i32) {
    let fraction =
        ((x - maximized[0]) as f64 / (maximized[2] - maximized[0]).max(1) as f64).clamp(0., 1.);
    let top_offset = (y - maximized[1]).clamp(0, title_height.round() as i32);
    (
        x - (fraction * restored_width as f64).round() as i32,
        y - top_offset,
    )
}

/// Integration check used only by the isolated window smoke test.
#[cfg(windows)]
pub fn verify_maximized_restore(handle: usize) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindowPlacement, WINDOWPLACEMENT};
    let hwnd = handle as HWND;
    let mut placement = WINDOWPLACEMENT {
        length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
        ..Default::default()
    };
    let mut maximized = RECT::default();
    let mut restored = RECT::default();
    // Safety: the smoke test calls this on its UI thread with its own live HWND.
    unsafe {
        if IsZoomed(hwnd) == 0
            || GetWindowPlacement(hwnd, &mut placement) == 0
            || GetWindowRect(hwnd, &mut maximized) == 0
        {
            return false;
        }
        let x = (maximized.left + maximized.right) / 2;
        let y = maximized.top + 24;
        if !restore_at(handle, x, y, 44.)
            || IsZoomed(hwnd) != 0
            || GetWindowRect(hwnd, &mut restored) == 0
        {
            return false;
        }
        let expected = placement.rcNormalPosition;
        (restored.right - restored.left - (expected.right - expected.left)).abs() <= 2
            && (restored.bottom - restored.top - (expected.bottom - expected.top)).abs() <= 2
            && (x - (restored.left + restored.right) / 2).abs() <= 1
            && y >= restored.top
            && y <= restored.top + 44
    }
}
#[cfg(not(windows))]
pub fn verify_maximized_restore(_handle: usize) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[cfg(windows)]
    fn native_drag_message_packs_coordinates_instead_of_passing_a_pointer() {
        for (x, y) in [(960, 24), (-1400, 32), (2800, -120)] {
            let packed = packed_point(x, y);
            assert_eq!(packed as i16 as i32, x);
            assert_eq!((packed >> 16) as i16 as i32, y);
        }
    }
    #[test]
    fn restore_keeps_the_pointer_over_the_title_bar_on_any_monitor() {
        assert_eq!(
            anchored_position([0, 0, 1920, 1080], 1000, 960, 24, 44.),
            (460, 0)
        );
        assert_eq!(
            anchored_position([-1920, 0, 0, 1080], 1000, -960, 24, 44.),
            (-1460, 0)
        );
        assert_eq!(
            anchored_position([0, 0, 1920, 1080], 1000, 480, 24, 44.),
            (230, 0)
        );
    }
}
