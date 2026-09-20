#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

#[cfg(windows)]
mod application;
#[cfg(windows)]
mod audio;
#[cfg(windows)]
mod audio_events;
#[cfg(windows)]
mod cursor_target;
#[cfg(windows)]
mod locator;
#[cfg(windows)]
mod process_identity;
#[cfg(windows)]
mod route_store;
#[cfg(windows)]
mod routing;
#[cfg(windows)]
mod ui;
#[cfg(windows)]
mod volume;

#[cfg(windows)]
fn main() {
    if let Err(error) = run() {
        report_fatal_error(&error.to_string());
        std::process::exit(1);
    }
}

#[cfg(windows)]
fn report_fatal_error(error: &str) {
    use crate::ui::i18n::{self, LanguagePreference, Text};

    let locale = LanguagePreference::load_default()
        .unwrap_or_else(|_| LanguagePreference::automatic())
        .locale();
    let localized = format!("{}\n\n{error}", i18n::text(locale, Text::CouldNotStart));
    eprintln!("{localized}");

    #[cfg(not(debug_assertions))]
    {
        use windows::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW};
        use windows::core::{PCWSTR, w};

        let message = localized
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        // SAFETY: Both UTF-16 strings remain valid for this synchronous call.
        let _ = unsafe {
            MessageBoxW(
                None,
                PCWSTR(message.as_ptr()),
                w!("Adufa"),
                MB_OK | MB_ICONERROR,
            )
        };
    }
}

#[cfg(not(windows))]
fn main() {
    eprintln!("router-windows only runs on Windows");
    std::process::exit(1);
}

#[cfg(windows)]
fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let observed = audio::observe()?;
    let (runtime, model) =
        application::ApplicationRuntime::start(observed).map_err(std::io::Error::other)?;
    let runtime = std::rc::Rc::new(std::cell::RefCell::new(runtime));
    let route_runtime = runtime.clone();
    let volume_runtime = runtime.clone();
    let refresh_runtime = runtime;
    ui::run_with_router(
        model,
        move |request| route_runtime.borrow_mut().route(request),
        move |request| volume_runtime.borrow_mut().set_volume(request),
        move || {
            let observed = audio::observe().map_err(|error| error.to_string())?;
            Ok(refresh_runtime.borrow_mut().refresh(observed))
        },
    )
}
