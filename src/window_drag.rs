use windows_sys::Win32::{
    Foundation::{HWND, POINT, RECT},
    UI::WindowsAndMessaging::{
        GetCursorPos, GetWindowRect, IsZoomed, SW_RESTORE, SWP_NOACTIVATE, SWP_NOSIZE,
        SWP_NOZORDER, SetWindowPos, ShowWindow,
    },
};

/// Borderless Windows windows do not reliably restore themselves on caption drag.
/// Restore synchronously, anchor the normal-sized window under the pointer, then
/// let winit start the same native move operation that works at normal size.
pub fn prepare(handle: usize, title_height: f32) {
    let hwnd = handle as HWND;
    let mut cursor = POINT { x: 0, y: 0 };
    // Safety: the HWND is borrowed from this app's live creation context. These
    // synchronous calls retain no pointers to our stack-owned coordinates.
    unsafe {
        if IsZoomed(hwnd) == 0 || GetCursorPos(&mut cursor) == 0 {
            return;
        }
    }
    restore_at(handle, cursor.x, cursor.y, title_height);
}

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

#[cfg(test)]
mod tests {
    use super::*;
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
