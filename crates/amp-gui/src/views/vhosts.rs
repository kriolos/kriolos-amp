use crate::i18n::I18n;
use egui::{Color32, RichText, Ui};

pub fn render_vhosts(ui: &mut Ui, i18n: &I18n) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.heading(RichText::new("🌐 ").size(20.0));
        ui.heading(RichText::new(i18n.t("vhosts")).strong().size(20.0));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(i18n.t("add_vhost")).clicked() {}
        });
    });

    ui.add_space(16.0);

    egui::Frame::group(ui.style())
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("localhost").strong().size(16.0));
                ui.label(RichText::new("(Default)").color(Color32::GRAY));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(i18n.t("open_in_browser")).clicked() {
                        let _ = open::that("http://localhost:8080");
                    }
                });
            });

            ui.add_space(6.0);
            ui.label(format!("{}: www", i18n.t("doc_root")));
            ui.label(format!("{}: 8080 | SSL: 8443", i18n.t("port")));
        });

    ui.add_space(12.0);
    ui.label(
        RichText::new(i18n.t("vhosts_info"))
            .color(Color32::GRAY)
            .italics(),
    );
}
