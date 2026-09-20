//! Native Windows presentation boundary.
//!
//! This module owns only presentation state and Win32 window resources. Audio
//! discovery and routing policy remain behind their existing module boundaries.

pub(crate) mod hotkeys;
pub(crate) mod i18n;
mod icons;
mod model;
mod motion;
mod selector;
mod startup;
mod symbols;
mod taskbar_quick_access;
mod theme;
mod tray;
mod window;

pub use model::{PopupModel, RouteRequest, VolumeRequest};

type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// Runs the popup and delegates each user routing choice to the platform layer.
///
/// Presentation state changes only after the callback reports success. This
/// prevents the UI from claiming a route that Windows rejected.
pub fn run_with_router<F, V, R>(
    model: PopupModel,
    route: F,
    set_volume: V,
    refresh: R,
) -> Result<(), BoxError>
where
    F: FnMut(RouteRequest) -> Result<(), String> + 'static,
    V: FnMut(VolumeRequest) -> Result<(), String> + 'static,
    R: FnMut() -> Result<PopupModel, String> + 'static,
{
    window::run(
        model,
        Box::new(route),
        Box::new(set_volume),
        Box::new(refresh),
    )
}
