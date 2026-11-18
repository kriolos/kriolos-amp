use crate::i18n::I18n;
use amp_core::PortChecker;
use egui::{Color32, RichText, Ui};

pub struct PortIndicator;

impl PortIndicator {
    pub fn show(ui: &mut Ui, i18n: &I18n, name: &str, port: u16) {
        if port == 0 {
            return;
        }

        let in_use = PortChecker::is_port_in_use(port);
        let (color, status_text) = if in_use {
            (Color32::from_rgb(34, 197, 94), i18n.t("port_in_use"))
        } else {
            (Color32::from_rgb(148, 163, 184), i18n.t("port_available"))
        };

        ui.horizontal(|ui| {
            ui.label(format!("{}:", name));
            ui.label(RichText::new(format!(":{}", port)).strong());
            ui.label(RichText::new(format!("({})", status_text)).color(color));
        });
    }
}
