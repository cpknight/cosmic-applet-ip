// SPDX-License-Identifier: GPL-3.0-only

mod app;
mod config;
mod i18n;
mod network;

fn main() -> cosmic::iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn".into()),
        )
        .init();

    // Load system locale for i18n.
    let requested = i18n_embed::DesktopLanguageRequester::requested_languages();
    i18n::init(&requested);

    cosmic::applet::run::<app::Window>(())
}
