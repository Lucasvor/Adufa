//! Shared Windows iconography for the compact popup surfaces.
//!
//! Standard actions use one operating-system symbol family instead of mixing
//! hand-drawn GDI shapes. Segoe Fluent Icons is vector-backed by Windows, so
//! the glyphs stay crisp at every supported DPI.

use windows::Win32::Foundation::{COLORREF, POINT, RECT};
use windows::Win32::Graphics::Gdi::{
    CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, CreateFontW, CreatePen, CreateSolidBrush,
    DEFAULT_CHARSET, DEFAULT_PITCH, DT_CENTER, DT_SINGLELINE, DT_VCENTER, DeleteObject, DrawTextW,
    Ellipse, FW_NORMAL, HDC, HGDIOBJ, OUT_DEFAULT_PRECIS, PS_SOLID, Polyline, SelectObject,
    SetBkMode, SetTextColor, TRANSPARENT,
};
use windows::core::w;

#[derive(Clone, Copy)]
pub enum Symbol {
    Volume,
    Mute,
    Settings,
    SoundLocator,
    Power,
    ChevronRight,
    Back,
    Check,
    Application,
    OpenExternal,
}

impl Symbol {
    const fn glyph(self) -> char {
        match self {
            Self::Volume => '\u{E767}',
            Self::Mute => '\u{E74F}',
            Self::Settings => '\u{E713}',
            // Sound Locator has a purpose-built mark rather than a font glyph.
            Self::SoundLocator => '\0',
            Self::Power => '\u{E7E8}',
            Self::ChevronRight => '\u{E76C}',
            Self::Back => '\u{E72B}',
            Self::Check => '\u{E73E}',
            Self::Application => '\u{E71D}',
            Self::OpenExternal => '\u{E8A7}',
        }
    }
}

/// Draws a centered Fluent symbol in the supplied logical rectangle.
pub fn draw(dc: HDC, symbol: Symbol, rect: RECT, logical_size: i32, color: COLORREF, dpi: u32) {
    if matches!(symbol, Symbol::SoundLocator) {
        draw_sound_locator(dc, rect, logical_size, color, dpi);
        return;
    }

    let height = scale(logical_size, dpi);
    // `CreateFontW` resolves to Segoe MDL2 Assets on older supported systems if
    // Segoe Fluent Icons is unavailable; both families share these codepoints.
    let font = unsafe {
        CreateFontW(
            -height,
            0,
            0,
            0,
            FW_NORMAL.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            DEFAULT_PITCH.0 as u32,
            w!("Segoe Fluent Icons"),
        )
    };
    let previous = unsafe { SelectObject(dc, HGDIOBJ(font.0)) };
    unsafe {
        SetBkMode(dc, TRANSPARENT);
        SetTextColor(dc, color);
    }
    let mut glyph = [symbol.glyph() as u16];
    let mut rect = rect;
    unsafe {
        DrawTextW(
            dc,
            &mut glyph,
            &mut rect,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );
        SelectObject(dc, previous);
        let _ = DeleteObject(HGDIOBJ(font.0));
    }
}

/// Draws a compact source point with two outgoing sound waves.
///
/// The mark stays meaningful without Windows-specific icon fonts, so macOS and
/// Linux shells can reproduce the same geometry with their native renderer.
fn draw_sound_locator(dc: HDC, rect: RECT, logical_size: i32, color: COLORREF, dpi: u32) {
    let size = scale(logical_size, dpi);
    let left = rect.left + (rect.right - rect.left - size) / 2;
    let top = rect.top + (rect.bottom - rect.top - size) / 2;
    let point = |x: i32, y: i32| POINT {
        x: left + scale(x, dpi),
        y: top + scale(y, dpi),
    };

    let stroke = scale(2, dpi).max(1);
    let pen = unsafe { CreatePen(PS_SOLID, stroke, color) };
    let brush = unsafe { CreateSolidBrush(color) };
    let previous = unsafe { SelectObject(dc, HGDIOBJ(pen.0)) };
    let previous_brush = unsafe { SelectObject(dc, HGDIOBJ(brush.0)) };
    let center = logical_size / 2;
    let inner_wave = [point(6, 5), point(9, center), point(6, 12)];
    let outer_wave = [point(10, 2), point(15, center), point(10, 15)];

    unsafe {
        let _ = Ellipse(
            dc,
            point(1, center - 2).x,
            point(1, center - 2).y,
            point(5, center + 2).x,
            point(5, center + 2).y,
        );
        let _ = Polyline(dc, &inner_wave);
        let _ = Polyline(dc, &outer_wave);
        SelectObject(dc, previous_brush);
        SelectObject(dc, previous);
        let _ = DeleteObject(HGDIOBJ(brush.0));
        let _ = DeleteObject(HGDIOBJ(pen.0));
    }
}

const fn scale(value: i32, dpi: u32) -> i32 {
    value * dpi as i32 / 96
}
