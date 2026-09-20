use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::UI::WindowsAndMessaging::{
    GA_ROOT, GetAncestor, GetCursorPos, GetWindowThreadProcessId, WindowFromPoint,
};

/// The root application window currently underneath the system cursor.
///
/// This is intentionally presentation-neutral. Consumers can match
/// `process_id` against their own application rows without coupling native
/// window discovery to a particular UI model.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CursorTarget {
    pub cursor_position: POINT,
    pub window: HWND,
    pub process_id: u32,
}

/// Resolves the top-level window and owning process underneath the cursor.
///
/// `None` is expected when the cursor is not over a window or when that window
/// disappears between the individual Win32 queries.
pub fn resolve() -> Option<CursorTarget> {
    let mut cursor_position = POINT::default();
    // SAFETY: cursor_position points to valid writable storage for the duration
    // of the call. GetCursorPos does not retain the pointer.
    unsafe { GetCursorPos(&mut cursor_position).ok()? };

    // SAFETY: cursor_position was initialized by GetCursorPos. WindowFromPoint
    // accepts the POINT by value and does not retain any borrowed data.
    let window_at_cursor = unsafe { WindowFromPoint(cursor_position) };
    if window_at_cursor.0.is_null() {
        return None;
    }

    // SAFETY: window_at_cursor was returned by WindowFromPoint. Handles can
    // become stale asynchronously; GetAncestor reports that case with null.
    let root_window = unsafe { GetAncestor(window_at_cursor, GA_ROOT) };
    if root_window.0.is_null() {
        return None;
    }

    let mut process_id = 0;
    // SAFETY: root_window is used only as an opaque handle and process_id points
    // to valid writable storage. The API does not retain either argument.
    let thread_id = unsafe { GetWindowThreadProcessId(root_window, Some(&mut process_id)) };
    if thread_id == 0 || process_id == 0 {
        return None;
    }

    Some(CursorTarget {
        cursor_position,
        window: root_window,
        process_id,
    })
}

/// Finds the first application row that owns `process_id`.
///
/// Passing only PID slices keeps this helper independent from `PopupModel`.
/// A caller can adapt its rows with
/// `model.applications.iter().map(|row| row.process_ids.as_slice())`.
pub fn application_index_for_process<'a>(
    process_id: u32,
    application_process_ids: impl IntoIterator<Item = &'a [u32]>,
) -> Option<usize> {
    application_process_ids
        .into_iter()
        .position(|process_ids| process_ids.contains(&process_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_process_to_its_application_row() {
        let rows: &[&[u32]] = &[&[11, 12], &[20], &[30, 31]];

        assert_eq!(
            application_index_for_process(20, rows.iter().copied()),
            Some(1)
        );
    }

    #[test]
    fn returns_none_when_process_is_not_in_the_visible_rows() {
        let rows: &[&[u32]] = &[&[11, 12], &[20], &[]];

        assert_eq!(
            application_index_for_process(99, rows.iter().copied()),
            None
        );
    }

    #[test]
    fn first_matching_row_wins_for_duplicate_process_data() {
        let rows: &[&[u32]] = &[&[42], &[7, 42]];

        assert_eq!(
            application_index_for_process(42, rows.iter().copied()),
            Some(0)
        );
    }
}
