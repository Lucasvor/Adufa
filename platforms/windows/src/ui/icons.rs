use std::mem::size_of;

use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateBitmap, CreateDIBSection, DIB_RGB_COLORS,
    DeleteObject, HDC, HGDIOBJ,
};
use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
use windows::Win32::UI::Controls::{IImageList, ILD_TRANSPARENT};
use windows::Win32::UI::Shell::{
    SHDefExtractIconW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SHGFI_SYSICONINDEX,
    SHGetFileInfoW, SHGetImageList, SHIL_EXTRALARGE, SHIL_JUMBO, SHIL_LARGE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateIconIndirect, DI_NORMAL, DestroyIcon, DrawIconEx, HICON, ICONINFO,
};
use windows::core::{BOOL, PCWSTR};

const TRAY_ICON_SIZE: usize = 16;
const TRAY_BITMAP_SIZE: usize = 32;
const TRAY_MASK_BYTES: usize = (TRAY_BITMAP_SIZE / 8) * TRAY_BITMAP_SIZE;
const TRAY_FOREGROUND: u32 = 0xFFF7_F8FA;
const TRAY_ACCENT: u32 = 0xFF60_9CFF;

// Pixel-aligned optical master of Adufa's route gate. The selected route is
// blue, the stopped alternate is neutral, and the white pixel cluster keeps the
// decision point legible at 16 px. Windows receives a 32-bit ARGB icon so the
// notification area keeps its transparency.
const TRAY_GLYPH: [&str; TRAY_ICON_SIZE] = [
    "................",
    "................",
    "..........bbbb..",
    ".........bbbbb..",
    "........bbb.....",
    ".......bbb......",
    "..bbbbbww.......",
    "..bbbbbwwg......",
    ".......gggg.....",
    "........gggg....",
    ".........g.g....",
    "..........gg....",
    "..........gg....",
    "................",
    "................",
    "................",
];

/// An icon returned by the Windows Shell and released with `DestroyIcon`.
pub struct OwnedIcon(HICON);

impl OwnedIcon {
    /// Loads the best Shell image-list icon for a logical size and DPI.
    ///
    /// The Shell's small-icon path is deliberately avoided: it only returns a
    /// 16-pixel bitmap, which becomes visibly pixelated when the UI draws it at
    /// 24 DIP (or larger on a high-DPI display).
    pub fn from_path_at_dpi(path: &str, logical_size: i32, dpi: u32) -> Option<Self> {
        let path = wide(path);
        Self::from_wide_path_at_dpi(&path, logical_size, dpi)
    }

    /// Creates Adufa's dedicated 32-bit notification-area icon.
    pub fn adufa_tray() -> Option<Self> {
        let pixels = tray_icon_pixels();
        let bitmap_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: TRAY_BITMAP_SIZE as i32,
                // A negative height creates a top-down DIB, matching the row
                // order produced by `tray_icon_pixels`.
                biHeight: -(TRAY_BITMAP_SIZE as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut dib_bits = std::ptr::null_mut();

        // SAFETY: Windows allocates the DIB storage and returns it through
        // `dib_bits`; the buffer is exactly 32 x 32 x 4 bytes.
        let color = unsafe {
            CreateDIBSection(None, &bitmap_info, DIB_RGB_COLORS, &mut dib_bits, None, 0).ok()?
        };
        if color.0.is_null() || dib_bits.is_null() {
            if !color.0.is_null() {
                let _ = unsafe { DeleteObject(HGDIOBJ(color.0)) };
            }
            return None;
        }

        // SAFETY: `dib_bits` points to the writable DIB storage described
        // above, and both source and destination contain exactly 1024 u32s.
        unsafe {
            std::ptr::copy_nonoverlapping(pixels.as_ptr(), dib_bits.cast(), pixels.len());
        }

        let mask_bits = [0u8; TRAY_MASK_BYTES];
        // SAFETY: the mask data covers every word-aligned row of the 32 x 32
        // monochrome bitmap and remains alive for the synchronous call.
        let mask = unsafe {
            CreateBitmap(
                TRAY_BITMAP_SIZE as i32,
                TRAY_BITMAP_SIZE as i32,
                1,
                1,
                Some(mask_bits.as_ptr().cast()),
            )
        };
        if mask.0.is_null() {
            let _ = unsafe { DeleteObject(HGDIOBJ(color.0)) };
            return None;
        }

        let icon_info = ICONINFO {
            fIcon: BOOL::from(true),
            hbmMask: mask,
            hbmColor: color,
            ..Default::default()
        };
        // CreateIconIndirect copies the bitmap data, so both temporary GDI
        // bitmaps can be released immediately after the call.
        let icon = unsafe { CreateIconIndirect(&icon_info).ok() };
        let _ = unsafe { DeleteObject(HGDIOBJ(mask.0)) };
        let _ = unsafe { DeleteObject(HGDIOBJ(color.0)) };
        icon.map(Self)
    }

    fn from_wide_path_at_dpi(path: &[u16], logical_size: i32, dpi: u32) -> Option<Self> {
        let source_size = physical_size(logical_size, dpi);

        exact_resource_icon(path, source_size)
            .or_else(|| shell_image_list_icon(path, source_size))
            .or_else(|| shell_large_icon(path))
    }

    pub fn draw(&self, dc: HDC, x: i32, y: i32, size: i32) {
        // SAFETY: the HICON is owned by this value and remains alive for the
        // synchronous draw; the paint DC belongs to the active WM_PAINT cycle.
        let _ = unsafe { DrawIconEx(dc, x, y, self.0, size, size, 0, None, DI_NORMAL) };
    }

    /// Borrows the native handle for APIs such as `Shell_NotifyIconW`.
    /// Ownership remains with this value.
    pub fn handle(&self) -> HICON {
        self.0
    }
}

fn exact_resource_icon(path: &[u16], source_size: i32) -> Option<OwnedIcon> {
    let mut icon = HICON::default();
    // SAFETY: The null-terminated path remains valid for the call and the
    // returned icon is caller-owned. The low word requests the exact large
    // icon size, letting Windows choose the best embedded resolution.
    let result = unsafe {
        SHDefExtractIconW(
            PCWSTR(path.as_ptr()),
            0,
            0,
            Some(&mut icon),
            None,
            source_size as u32,
        )
    };
    (result.is_ok() && !icon.0.is_null()).then_some(OwnedIcon(icon))
}

fn shell_image_list_icon(path: &[u16], source_size: i32) -> Option<OwnedIcon> {
    let mut info = SHFILEINFOW::default();
    // SAFETY: `path` is null-terminated and remains alive for the call. We only
    // request the shared system-image-list index, so no HICON is transferred.
    let result = unsafe {
        SHGetFileInfoW(
            PCWSTR(path.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut info),
            size_of::<SHFILEINFOW>() as u32,
            SHGFI_SYSICONINDEX | SHGFI_LARGEICON,
        )
    };
    if result == 0 {
        return None;
    }

    // Choose an image list whose source bitmap is at least as large as the
    // requested physical size. This prevents DrawIconEx from magnifying a 16px
    // Shell icon on 125-200% displays.
    let preferred = match source_size {
        ..=32 => SHIL_LARGE,
        33..=48 => SHIL_EXTRALARGE,
        _ => SHIL_JUMBO,
    };
    let tiers = match preferred {
        SHIL_LARGE => [SHIL_LARGE, SHIL_EXTRALARGE, SHIL_JUMBO],
        SHIL_EXTRALARGE => [SHIL_EXTRALARGE, SHIL_JUMBO, SHIL_LARGE],
        _ => [SHIL_JUMBO, SHIL_EXTRALARGE, SHIL_LARGE],
    };

    for tier in tiers {
        // SAFETY: SHGetImageList creates a valid COM interface on success;
        // IImageList::GetIcon returns a caller-owned HICON.
        let icon = unsafe {
            SHGetImageList::<IImageList>(tier as i32)
                .and_then(|images| images.GetIcon(info.iIcon, ILD_TRANSPARENT.0))
                .ok()
        };
        if let Some(icon) = icon.filter(|icon| !icon.0.is_null()) {
            return Some(OwnedIcon(icon));
        }
    }

    None
}

fn shell_large_icon(path: &[u16]) -> Option<OwnedIcon> {
    let mut info = SHFILEINFOW::default();
    // SAFETY: `path` is null-terminated and remains alive for the call. Shell32
    // initializes `info`; SHGFI_ICON transfers ownership of the returned HICON.
    let result = unsafe {
        SHGetFileInfoW(
            PCWSTR(path.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut info),
            size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        )
    };
    (result != 0 && !info.hIcon.0.is_null()).then_some(OwnedIcon(info.hIcon))
}

fn tray_icon_pixels() -> Vec<u32> {
    let mut pixels = vec![0; TRAY_BITMAP_SIZE * TRAY_BITMAP_SIZE];
    for (y, row) in TRAY_GLYPH.iter().enumerate() {
        for (x, pixel) in row.bytes().enumerate() {
            let color = match pixel {
                b'b' => TRAY_ACCENT,
                b'w' => TRAY_FOREGROUND,
                b'g' => 0xFF7C_8796,
                _ => continue,
            };
            for scale_y in 0..2 {
                for scale_x in 0..2 {
                    let target_x = x * 2 + scale_x;
                    let target_y = y * 2 + scale_y;
                    pixels[target_y * TRAY_BITMAP_SIZE + target_x] = color;
                }
            }
        }
    }
    pixels
}

fn physical_size(logical_size: i32, dpi: u32) -> i32 {
    let logical_size = logical_size.max(1) as i64;
    let dpi = dpi.max(96) as i64;
    ((logical_size * dpi + 95) / 96).min(i32::MAX as i64) as i32
}

impl Drop for OwnedIcon {
    fn drop(&mut self) {
        // SAFETY: every constructor stores only a caller-owned HICON returned by
        // the Shell/image list, and this value releases that handle exactly once.
        let _ = unsafe { DestroyIcon(self.0) };
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain([0]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tray_icon_contains_transparent_foreground_and_accent_pixels() {
        let pixels = tray_icon_pixels();
        assert_eq!(pixels.len(), TRAY_BITMAP_SIZE * TRAY_BITMAP_SIZE);
        assert!(pixels.contains(&0));
        assert!(pixels.iter().any(|pixel| pixel >> 24 == 0xFF));
        assert!(pixels.contains(&TRAY_ACCENT));
        assert!(pixels.contains(&TRAY_FOREGROUND));
        assert!(pixels.contains(&0xFF7C_8796));
    }

    #[test]
    fn tray_icon_can_be_materialized_as_a_windows_icon() {
        assert!(OwnedIcon::adufa_tray().is_some());
    }
}
