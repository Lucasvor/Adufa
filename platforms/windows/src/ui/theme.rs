use windows::Win32::Foundation::COLORREF;

/// Semantic color roles for the compact native popup.
///
/// Keeping the palette outside paint code lets a future light/system theme
/// replace colors without changing geometry or interaction behavior.
#[derive(Clone, Copy)]
pub struct Theme {
    pub surface: COLORREF,
    pub hover: COLORREF,
    pub pressed: COLORREF,
    pub rule: COLORREF,
    pub text: COLORREF,
    pub secondary_text: COLORREF,
    pub accent: COLORREF,
    /// Low-emphasis accent surface used for active, non-destructive modes.
    pub accent_soft: COLORREF,
    /// Quiet track behind live signal meters.
    pub meter_track: COLORREF,
}

impl Theme {
    pub const DARK: Self = Self {
        surface: rgb(30, 32, 36),
        hover: rgb(45, 48, 54),
        pressed: rgb(54, 58, 65),
        rule: rgb(63, 67, 74),
        text: rgb(247, 248, 250),
        secondary_text: rgb(189, 194, 203),
        accent: rgb(96, 156, 255),
        accent_soft: rgb(32, 41, 53),
        meter_track: rgb(46, 50, 57),
    };
}

const fn rgb(red: u8, green: u8, blue: u8) -> COLORREF {
    COLORREF(red as u32 | ((green as u32) << 8) | ((blue as u32) << 16))
}
