// SPDX-License-Identifier: GPL-3.0-only

use std::sync::LazyLock;
use std::time::Duration;

use cosmic::app::Task;
use cosmic::cosmic_config::CosmicConfigEntry;
use cosmic::iced::{
    Alignment, Length, Limits, Subscription,
    platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup},
    time,
    window::Id as WindowId,
};
use cosmic::prelude::*;
use cosmic::widget::{self, Id as WidgetId};

/// Stable ID used by `autosize` to ask the Wayland compositor to grow the
/// applet window to fit the icon + IP text.
static AUTOSIZE_MAIN_ID: LazyLock<WidgetId> =
    LazyLock::new(|| WidgetId::new("cosmic-applet-ip-autosize"));

use crate::config::{IpAppletConfig, Selection};
use crate::fl;
use crate::network::{self, Interface};

/// The reverse-DNS application ID — also the desktop-file basename and the
/// `cosmic-config` namespace for persisted settings.
pub const APP_ID: &str = "com.cpknight.CosmicAppletIp";

pub struct Window {
    core: cosmic::app::Core,
    popup: Option<WindowId>,
    interfaces: Vec<Interface>,
    /// Index into the cycle list `[Auto, iface_0, iface_1, ...]`.
    /// 0 always means "Auto".
    cycle_index: usize,
    config: IpAppletConfig,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// User clicked the panel button — advance the cycle & open the popup.
    CycleClicked,
    /// 2-second poll tick.
    Tick,
    /// Popup was closed (user clicked away, pressed Esc, …).
    PopupClosed(WindowId),
    /// Reset selection back to `Auto`.
    ResetAuto,
    /// Pick a specific interface directly from the popup list.
    SelectInterface(String),
}

impl Window {
    /// Currently-active `Interface` (resolving `Auto` to the default-route
    /// interface when possible).
    fn active_interface(&self) -> Option<&Interface> {
        match &self.config.selection {
            Selection::Auto => {
                if let Some(name) = network::default_interface_name() {
                    if let Some(iface) = self.interfaces.iter().find(|i| i.name == name) {
                        return Some(iface);
                    }
                }
                // Fallback: first non-loopback interface with an address
                self.interfaces
                    .iter()
                    .find(|i| !i.is_loopback && i.has_address)
                    .or_else(|| self.interfaces.iter().find(|i| !i.is_loopback))
            }
            Selection::Pinned(name) => self.interfaces.iter().find(|i| &i.name == name),
        }
    }

    fn refresh_interfaces(&mut self) {
        self.interfaces = network::list_interfaces();
        self.cycle_index = match &self.config.selection {
            Selection::Auto => 0,
            Selection::Pinned(name) => self
                .interfaces
                .iter()
                .position(|i| &i.name == name)
                .map(|p| p + 1)
                .unwrap_or(0),
        };
    }

    fn persist_config(&self) {
        if let Ok(ctx) = cosmic::cosmic_config::Config::new(APP_ID, IpAppletConfig::VERSION) {
            if let Err(err) = self.config.write_entry(&ctx) {
                tracing::warn!(?err, "failed to persist IP applet config");
            }
        }
    }

    fn ensure_popup(&mut self) -> Task<Message> {
        if self.popup.is_some() {
            return Task::none();
        }
        let Some(parent) = self.core.main_window_id() else {
            return Task::none();
        };
        let new_id = WindowId::unique();
        self.popup = Some(new_id);
        let mut settings = self
            .core
            .applet
            .get_popup_settings(parent, new_id, None, None, None);
        settings.positioner.size_limits = Limits::NONE
            .max_width(380.0)
            .min_width(260.0)
            .min_height(100.0)
            .max_height(600.0);
        get_popup(settings)
    }

    fn panel_label(&self) -> String {
        if let Some(iface) = self.active_interface() {
            if let Some(ip) = iface.primary_display() {
                return ip.to_string();
            }
        }
        "—".to_string()
    }
}

impl cosmic::Application for Window {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &cosmic::app::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::app::Core {
        &mut self.core
    }

    fn init(core: cosmic::app::Core, _flags: ()) -> (Self, Task<Message>) {
        let config = IpAppletConfig::load();
        let mut window = Self {
            core,
            popup: None,
            interfaces: Vec::new(),
            cycle_index: 0,
            config,
        };
        window.refresh_interfaces();
        (window, Task::none())
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }

    fn on_close_requested(&self, id: WindowId) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_secs(2)).map(|_| Message::Tick)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.refresh_interfaces();
                Task::none()
            }
            Message::CycleClicked => {
                self.refresh_interfaces();
                let total = self.interfaces.len() + 1; // +1 for Auto
                if total > 1 {
                    self.cycle_index = (self.cycle_index + 1) % total;
                }
                self.config.selection = match self.cycle_index {
                    0 => Selection::Auto,
                    n => Selection::Pinned(self.interfaces[n - 1].name.clone()),
                };
                self.persist_config();
                self.ensure_popup()
            }
            Message::ResetAuto => {
                self.config.selection = Selection::Auto;
                self.cycle_index = 0;
                self.persist_config();
                Task::none()
            }
            Message::SelectInterface(name) => {
                if let Some(pos) = self.interfaces.iter().position(|i| i.name == name) {
                    self.cycle_index = pos + 1;
                    self.config.selection = Selection::Pinned(name);
                    self.persist_config();
                }
                Task::none()
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
                // Also destroy it on the Wayland side if it's still around.
                let _ = destroy_popup::<Message>(id);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let ip_label = self.panel_label();
        let horizontal = self.core.applet.is_horizontal();

        // On vertical panels (Left/Right), showing an IP string side-by-side
        // looks terrible; fall back to the plain icon button the same way
        // the time/network applets do.
        if !self.config.show_label || ip_label == "—" || !horizontal {
            return self
                .core
                .applet
                .icon_button("network-wired-symbolic")
                .on_press(Message::CycleClicked)
                .into();
        }

        let (suggested_w, suggested_h) = self.core.applet.suggested_size(true);
        let (pad_major, pad_minor) = self.core.applet.suggested_padding(true);
        // Horizontal panel → major axis is horizontal padding.
        let (h_pad, v_pad) = (pad_major, pad_minor);
        let total_h = f32::from(suggested_h + 2 * v_pad);

        // Icon + size-aware text, vertically centered.
        //
        // `self.core.applet.text(...)` returns a panel-sized text widget that
        // matches the sizing the time / weather applets use, so our IP string
        // lines up with the neighbouring applets' text baselines.
        let row = widget::row::with_capacity(2)
            .spacing(4)
            .align_y(Alignment::Center)
            .push(
                widget::icon::from_name("network-wired-symbolic")
                    .symbolic(true)
                    .size(suggested_w),
            )
            .push(self.core.applet.text(ip_label));

        // `layer_container(...).center_y(Length::Fill)` centers the row
        // vertically within the button (matching `icon_button`'s behavior),
        // and pinning the button height to the full panel height keeps the
        // baseline aligned with the time/weather applets.
        let button = widget::button::custom(
            widget::layer_container(row)
                .center_y(Length::Fill)
                .height(Length::Fill),
        )
        .padding([0, h_pad])
        .height(Length::Fixed(total_h))
        .class(cosmic::theme::Button::AppletIcon)
        .on_press(Message::CycleClicked);

        // Let the Wayland compositor grow our applet's window to fit the
        // icon + IP text instead of clipping to the default icon-only width.
        widget::autosize::autosize(button, AUTOSIZE_MAIN_ID.clone())
            .min_width(total_h)
            .min_height(total_h)
            .into()
    }

    fn view_window(&self, _id: WindowId) -> Element<'_, Message> {
        let active = self.active_interface();

        let header: Element<'_, Message> = match (&self.config.selection, active) {
            (Selection::Auto, Some(iface)) => {
                widget::text::title4(fl!("auto-arrow", name = iface.name.clone())).into()
            }
            (Selection::Auto, None) => widget::text::title4(fl!("auto")).into(),
            (Selection::Pinned(name), _) => widget::text::title4(name.clone()).into(),
        };

        let mut details = widget::column::with_capacity(4).spacing(4);
        match active {
            Some(iface) => {
                let status = if iface.is_connected() {
                    fl!("connected")
                } else {
                    fl!("disconnected")
                };
                details = details.push(widget::settings::item(fl!("status"), widget::text(status)));
                if !iface.ipv4.is_empty() {
                    let v4 = iface
                        .ipv4
                        .iter()
                        .map(|a| a.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    details = details.push(widget::settings::item(fl!("ipv4"), widget::text(v4)));
                }
                if !iface.ipv6.is_empty() {
                    let v6 = iface
                        .ipv6
                        .iter()
                        .map(|a| a.to_string())
                        .collect::<Vec<_>>()
                        .join("\n");
                    details = details.push(widget::settings::item(fl!("ipv6"), widget::text(v6)));
                }
            }
            None => {
                details = details.push(widget::text(fl!("no-interfaces")));
            }
        }

        let mut pick = widget::column::with_capacity(self.interfaces.len() + 1).spacing(2);
        pick = pick.push(
            widget::button::text(fl!("reset-to-auto"))
                .width(Length::Fill)
                .on_press(Message::ResetAuto),
        );
        for iface in &self.interfaces {
            if iface.is_loopback {
                continue;
            }
            let marker = if iface.is_connected() { "●" } else { "○" };
            let label = format!(
                "{marker}  {name}  {ip}",
                name = iface.name,
                ip = iface
                    .primary_display()
                    .map(|a| a.to_string())
                    .unwrap_or_else(|| fl!("no-address")),
            );
            pick = pick.push(
                widget::button::text(label)
                    .width(Length::Fill)
                    .on_press(Message::SelectInterface(iface.name.clone())),
            );
        }

        let content = widget::column::with_capacity(5)
            .spacing(8)
            .padding(12)
            .push(header)
            .push(widget::divider::horizontal::default())
            .push(details)
            .push(widget::divider::horizontal::default())
            .push(pick);

        self.core.applet.popup_container(content).into()
    }
}
